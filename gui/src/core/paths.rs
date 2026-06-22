use std::path::{Path, PathBuf};

/// Resolves the root directory of zapret (the folder containing bin/, lists/, *.bat).
/// Strategy:
///   1. Directory of current_exe()
///   2. Walk up looking for a sibling `bin\winws.exe`
///   3. Fallback to current_exe parent
pub fn root_dir() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let start = exe.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."));

    let mut cur = start.clone();
    for _ in 0..6 {
        if cur.join("bin").join("winws.exe").exists() {
            return cur;
        }
        match cur.parent() {
            Some(p) => cur = p.to_path_buf(),
            None => break,
        }
    }
    start
}

pub fn bin_dir() -> PathBuf {
    root_dir().join("bin")
}

pub fn lists_dir() -> PathBuf {
    root_dir().join("lists")
}

pub fn utils_dir() -> PathBuf {
    root_dir().join("utils")
}

pub fn winws_exe() -> PathBuf {
    bin_dir().join("winws.exe")
}

/// Enumerates `general*.bat` files in the root, sorted by natural order.
pub fn strategy_bats() -> Vec<PathBuf> {
    let root = root_dir();
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&root) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    let lower = name.to_ascii_lowercase();
                    if lower.starts_with("general") && lower.ends_with(".bat") {
                        out.push(p);
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| natural_cmp(a.file_name().unwrap().to_string_lossy().as_ref(),
                                   b.file_name().unwrap().to_string_lossy().as_ref()));
    out
}

fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let mut ai = a.chars().peekable();
    let mut bi = b.chars().peekable();
    loop {
        match (ai.peek().copied(), bi.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(ca), Some(cb)) => {
                if ca.is_ascii_digit() && cb.is_ascii_digit() {
                    let mut na = String::new();
                    let mut nb = String::new();
                    while let Some(c) = ai.peek().copied() {
                        if c.is_ascii_digit() { na.push(c); ai.next(); } else { break; }
                    }
                    while let Some(c) = bi.peek().copied() {
                        if c.is_ascii_digit() { nb.push(c); bi.next(); } else { break; }
                    }
                    let xa: u64 = na.parse().unwrap_or(0);
                    let xb: u64 = nb.parse().unwrap_or(0);
                    match xa.cmp(&xb) {
                        Ordering::Equal => continue,
                        o => return o,
                    }
                } else {
                    let la = ca.to_ascii_lowercase();
                    let lb = cb.to_ascii_lowercase();
                    match la.cmp(&lb) {
                        Ordering::Equal => { ai.next(); bi.next(); }
                        o => return o,
                    }
                }
            }
        }
    }
}

pub fn hosts_file() -> PathBuf {
    let sysroot = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    PathBuf::from(sysroot).join("System32").join("drivers").join("etc").join("hosts")
}
