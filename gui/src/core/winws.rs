use crate::core::paths;
use anyhow::Context;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::io::{BufRead, BufReader};
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const MAX_LOG_LINES: usize = 4000;

#[derive(Default)]
pub struct LogBuffer {
    lines: Mutex<std::collections::VecDeque<String>>,
}

impl LogBuffer {
    pub fn push(&self, s: String) {
        let mut g = self.lines.lock().unwrap();
        if g.len() >= MAX_LOG_LINES {
            g.pop_front();
        }
        g.push_back(s);
    }

    pub fn snapshot(&self) -> Vec<String> {
        self.lines.lock().unwrap().iter().cloned().collect()
    }

    pub fn clear(&self) {
        self.lines.lock().unwrap().clear();
    }

    pub fn len(&self) -> usize {
        self.lines.lock().unwrap().len()
    }
}

pub struct Runner {
    child: Option<Child>,
    pub logs: Arc<LogBuffer>,
    pub current_strategy: Option<String>,
    _stop_tx: Option<Sender<()>>,
}

impl Default for Runner {
    fn default() -> Self {
        Self {
            child: None,
            logs: Arc::new(LogBuffer::default()),
            current_strategy: None,
            _stop_tx: None,
        }
    }
}

impl Runner {
    pub fn is_running(&mut self) -> bool {
        if let Some(child) = self.child.as_mut() {
            match child.try_wait() {
                Ok(None) => return true,
                _ => {
                    self.child = None;
                    self.current_strategy = None;
                    return false;
                }
            }
        }
        false
    }

    /// Returns true if an external winws.exe is running (not managed by us).
    #[allow(dead_code)]
    pub fn external_running() -> bool {
        super::service::is_process_running("winws.exe")
    }

    pub fn start(&mut self, strategy_name: &str, args: &[String]) -> anyhow::Result<()> {
        if self.is_running() {
            anyhow::bail!("Strategy is already running");
        }

        let exe = paths::winws_exe();
        if !exe.exists() {
            anyhow::bail!("winws.exe not found at {}", exe.display());
        }

        self.logs.push(format!("→ Starting strategy: {}", strategy_name));
        self.logs.push(format!("  exe: {}", exe.display()));
        self.logs.push(format!("  args ({}): {}", args.len(), args.join(" ")));

        let mut cmd = Command::new(&exe);
        cmd.args(args)
            .current_dir(paths::bin_dir())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW);

        let mut child = cmd.spawn().with_context(|| format!("failed to spawn {}", exe.display()))?;

        if let Some(stdout) = child.stdout.take() {
            let logs = self.logs.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(l) = line {
                        logs.push(l);
                    }
                }
            });
        }
        if let Some(stderr) = child.stderr.take() {
            let logs = self.logs.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(l) = line {
                        logs.push(format!("[err] {}", l));
                    }
                }
            });
        }

        self.child = Some(child);
        self.current_strategy = Some(strategy_name.to_string());
        Ok(())
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(mut child) = self.child.take() {
            self.logs.push("→ Stopping winws.exe ...".to_string());
            let _ = child.kill();
            let _ = child.wait();
        }
        // Safety net: ensure no detached winws.exe instances remain.
        let _ = Command::new("taskkill")
            .args(["/IM", "winws.exe", "/F"])
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        self.current_strategy = None;
        self.logs.push("✓ Stopped.".to_string());
        Ok(())
    }
}

/// Helper to spawn a quick external command silently and capture combined output.
pub fn run_silent(program: &str, args: &[&str]) -> anyhow::Result<(i32, String)> {
    let out = Command::new(program)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .with_context(|| format!("failed to run {}", program))?;
    let mut s = String::new();
    s.push_str(&String::from_utf8_lossy(&out.stdout));
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    Ok((out.status.code().unwrap_or(-1), s))
}

// Suppress unused warnings for the channel type used internally.
#[allow(dead_code)]
fn _ch_check() -> (Sender<()>, Receiver<()>) { unbounded() }
#[allow(dead_code)]
fn _path_check(_: &Path) {}
