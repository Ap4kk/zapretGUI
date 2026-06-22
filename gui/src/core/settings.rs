use crate::core::paths;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Disabled,
    Both,
    TcpOnly,
    UdpOnly,
}

impl GameMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Both => "TCP + UDP",
            Self::TcpOnly => "TCP only",
            Self::UdpOnly => "UDP only",
        }
    }
}

pub fn game_flag_file() -> PathBuf {
    paths::utils_dir().join("game_filter.enabled")
}

pub fn check_updates_flag_file() -> PathBuf {
    paths::utils_dir().join("check_updates.enabled")
}

pub fn read_game_mode() -> GameMode {
    let path = game_flag_file();
    if !path.exists() {
        return GameMode::Disabled;
    }
    match std::fs::read_to_string(&path) {
        Ok(s) => {
            let v = s.trim().to_ascii_lowercase();
            match v.as_str() {
                "all" => GameMode::Both,
                "tcp" => GameMode::TcpOnly,
                "udp" => GameMode::UdpOnly,
                _ => GameMode::Disabled,
            }
        }
        Err(_) => GameMode::Disabled,
    }
}

pub fn write_game_mode(mode: GameMode) -> Result<()> {
    let path = game_flag_file();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    match mode {
        GameMode::Disabled => {
            if path.exists() {
                std::fs::remove_file(&path)?;
            }
        }
        GameMode::Both => std::fs::write(&path, "all")?,
        GameMode::TcpOnly => std::fs::write(&path, "tcp")?,
        GameMode::UdpOnly => std::fs::write(&path, "udp")?,
    }
    Ok(())
}

pub fn read_auto_update() -> bool {
    check_updates_flag_file().exists()
}

pub fn write_auto_update(enabled: bool) -> Result<()> {
    let path = check_updates_flag_file();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    if enabled {
        if !path.exists() {
            std::fs::write(&path, "ENABLED")?;
        }
    } else if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpsetMode {
    None,
    Loaded,
    Any,
}

impl IpsetMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Loaded => "loaded",
            Self::Any => "any",
        }
    }
}

pub fn ipset_path() -> PathBuf {
    paths::lists_dir().join("ipset-all.txt")
}

pub fn ipset_backup_path() -> PathBuf {
    paths::lists_dir().join("ipset-all.txt.backup")
}

pub fn read_ipset_mode() -> IpsetMode {
    let path = ipset_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return IpsetMode::Any,
    };
    let non_empty: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    if non_empty.is_empty() {
        return IpsetMode::Any;
    }
    if non_empty.iter().any(|l| l.contains("203.0.113.113/32")) && non_empty.len() <= 2 {
        return IpsetMode::None;
    }
    IpsetMode::Loaded
}

/// Cycle to next IPSet mode (loaded -> none -> any -> loaded).
pub fn switch_ipset_mode() -> Result<IpsetMode> {
    let path = ipset_path();
    let backup = ipset_backup_path();
    let mode = read_ipset_mode();
    let next = match mode {
        IpsetMode::Loaded => {
            // Move current list to backup, write a stub.
            if backup.exists() {
                std::fs::remove_file(&backup)?;
            }
            std::fs::rename(&path, &backup).ok();
            std::fs::write(&path, "203.0.113.113/32\r\n")?;
            IpsetMode::None
        }
        IpsetMode::None => {
            std::fs::write(&path, "")?;
            IpsetMode::Any
        }
        IpsetMode::Any => {
            if backup.exists() {
                if path.exists() {
                    std::fs::remove_file(&path)?;
                }
                std::fs::rename(&backup, &path)?;
                IpsetMode::Loaded
            } else {
                return Err(anyhow::anyhow!(
                    "No backup to restore. Update the IPSet list first."
                ));
            }
        }
    };
    Ok(next)
}

pub fn ensure_user_lists() -> Result<()> {
    let dir = paths::lists_dir();
    std::fs::create_dir_all(&dir)?;
    let ipset_exclude = dir.join("ipset-exclude-user.txt");
    if !ipset_exclude.exists() {
        std::fs::write(&ipset_exclude, "203.0.113.113/32\r\n")?;
    }
    let list_general = dir.join("list-general-user.txt");
    if !list_general.exists() {
        std::fs::write(&list_general, "# Never leave this file empty\r\ndomain.example.abc\r\n")?;
    }
    let list_exclude = dir.join("list-exclude-user.txt");
    if !list_exclude.exists() {
        std::fs::write(&list_exclude, "domain.example.abc\r\n")?;
    }
    Ok(())
}
