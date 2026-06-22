use crate::core::paths;
use std::path::{Path, PathBuf};

/// Parsed strategy: arguments for winws.exe, with all variables expanded.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParsedStrategy {
    pub source: PathBuf,
    pub display_name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct GameFilter {
    pub tcp: &'static str,
    pub udp: &'static str,
}

impl GameFilter {
    pub fn from_settings(mode: crate::core::settings::GameMode) -> Self {
        use crate::core::settings::GameMode;
        match mode {
            GameMode::Disabled => GameFilter { tcp: "12", udp: "12" },
            GameMode::Both => GameFilter { tcp: "1024-65535", udp: "1024-65535" },
            GameMode::TcpOnly => GameFilter { tcp: "1024-65535", udp: "12" },
            GameMode::UdpOnly => GameFilter { tcp: "12", udp: "1024-65535" },
        }
    }
}

pub fn parse_bat(path: &Path, gf: GameFilter) -> anyhow::Result<ParsedStrategy> {
    let bytes = std::fs::read(path)?;
    // Many bats are UTF-8 with BOM; some can be system-cp. UTF-8 lossy is good enough here.
    let text = String::from_utf8_lossy(&bytes).into_owned();

    let bin = paths::bin_dir().to_string_lossy().to_string();
    let lists = paths::lists_dir().to_string_lossy().to_string();
    let bin = ensure_trailing_sep(&bin);
    let lists = ensure_trailing_sep(&lists);

    // Find the logical command line: from the line containing `winws.exe` until
    // a line that does NOT end with a `^` continuation.
    let mut joined = String::new();
    let mut capturing = false;
    let mut started = false;

    for raw_line in text.lines() {
        // Strip trailing CR if any (lines() already removes \n)
        let line = raw_line.trim_end_matches('\r');
        // Skip pure comment lines that start with ::
        let trimmed = line.trim_start();
        if !capturing && trimmed.starts_with("::") {
            continue;
        }

        if !capturing {
            if line.to_ascii_lowercase().contains("winws.exe") {
                capturing = true;
                started = true;
                // Find the position right after winws.exe to drop the executable part.
                if let Some(pos) = find_after_winws(line) {
                    let tail = &line[pos..];
                    joined.push_str(tail);
                } else {
                    joined.push_str(line);
                }
            }
            continue;
        }

        // capturing == true: append, removing line continuation backslash
        let mut piece = line.to_string();
        let has_continuation = piece.trim_end().ends_with('^');
        if has_continuation {
            // remove the caret at the end
            let trimmed_end = piece.trim_end();
            let new_len = trimmed_end.len().saturating_sub(1);
            piece.truncate(new_len);
        }
        joined.push(' ');
        joined.push_str(&piece);

        if !has_continuation {
            break;
        }
    }

    if !started {
        anyhow::bail!("winws.exe command not found in {}", path.display());
    }

    let tokens = tokenize(&joined);
    let mut args: Vec<String> = Vec::with_capacity(tokens.len());
    for tok in tokens {
        let mut t = tok;
        t = t.replace("%BIN%", &bin);
        t = t.replace("%LISTS%", &lists);
        t = t.replace("%GameFilterTCP%", gf.tcp);
        t = t.replace("%GameFilterUDP%", gf.udp);
        t = t.replace("%GameFilter%", "1024-65535");
        args.push(t);
    }

    let display_name = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "general".to_string());

    Ok(ParsedStrategy {
        source: path.to_path_buf(),
        display_name,
        args,
    })
}

fn ensure_trailing_sep(p: &str) -> String {
    if p.ends_with('\\') || p.ends_with('/') {
        p.to_string()
    } else {
        format!("{}\\", p)
    }
}

fn find_after_winws(line: &str) -> Option<usize> {
    // Find case-insensitive position of "winws.exe" and return byte index right after it,
    // skipping a possible trailing quote.
    let lower = line.to_ascii_lowercase();
    let needle = "winws.exe";
    let pos = lower.find(needle)?;
    let mut after = pos + needle.len();
    // skip closing quote if any
    if line.as_bytes().get(after) == Some(&b'"') {
        after += 1;
    }
    Some(after)
}

/// Simple shell-like tokenizer that respects double quotes.
/// Keeps the quotes intact for tokens that contain `=` followed by a quoted value,
/// e.g. --foo="C:\path with space\file.bin".
fn tokenize(input: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;

    for ch in input.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                // Drop the quote chars themselves — the resulting token is the raw value.
            }
            c if c.is_whitespace() && !in_quotes => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_basic() {
        let v = tokenize(r#"--a=1 --b "c:\path with space\f.txt" --c=2"#);
        assert_eq!(v, vec!["--a=1", "--b", r"c:\path with space\f.txt", "--c=2"]);
    }

    #[test]
    fn tokenize_quoted_eq() {
        let v = tokenize(r#"--foo="bar baz" --x=1"#);
        assert_eq!(v, vec!["--foo=bar baz", "--x=1"]);
    }
}
