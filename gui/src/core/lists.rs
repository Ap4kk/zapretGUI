use crate::core::paths;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub enum UserList {
    GeneralUser,
    ExcludeUser,
    IpsetAll,
    IpsetExcludeUser,
}

impl UserList {
    pub fn label(self) -> &'static str {
        match self {
            Self::GeneralUser => "list-general-user.txt",
            Self::ExcludeUser => "list-exclude-user.txt",
            Self::IpsetAll => "ipset-all.txt",
            Self::IpsetExcludeUser => "ipset-exclude-user.txt",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::GeneralUser => "Domains to add to the bypass on top of the bundled list",
            Self::ExcludeUser => "Domains to exclude from the bypass",
            Self::IpsetAll => "IP ranges to apply the bypass to (updatable from repo)",
            Self::IpsetExcludeUser => "IP ranges to exclude from the bypass",
        }
    }
    pub fn path(self) -> PathBuf {
        paths::lists_dir().join(self.label())
    }
}

pub fn read(list: UserList) -> Result<String> {
    let path = list.path();
    if !path.exists() {
        return Ok(String::new());
    }
    Ok(std::fs::read_to_string(path)?)
}

pub fn write(list: UserList, content: &str) -> Result<()> {
    let path = list.path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    // Normalize to CRLF so files behave well with notepad.
    let normalized: String = content.replace("\r\n", "\n").replace('\n', "\r\n");
    std::fs::write(path, normalized)?;
    Ok(())
}
