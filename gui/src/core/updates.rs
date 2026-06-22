use crate::core::paths;
use anyhow::{Context, Result};
use std::time::Duration;

const VERSION_URL: &str =
    "https://raw.githubusercontent.com/Flowseal/zapret-discord-youtube/main/.service/version.txt";
const IPSET_URL: &str =
    "https://raw.githubusercontent.com/Flowseal/zapret-discord-youtube/main/.service/ipset-service.txt";
const HOSTS_URL: &str =
    "https://raw.githubusercontent.com/Flowseal/zapret-discord-youtube/main/.service/hosts";

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(30))
        .user_agent("zapret-gui/0.1")
        .build()
}

pub fn local_version() -> Option<String> {
    let path = paths::root_dir().join(".service").join("version.txt");
    std::fs::read_to_string(&path).ok().map(|s| s.trim().to_string())
}

pub fn remote_version() -> Result<String> {
    let body = agent().get(VERSION_URL).call()?.into_string()?;
    Ok(body.trim().to_string())
}

pub fn update_ipset() -> Result<usize> {
    let body = agent().get(IPSET_URL).call()?.into_string()?;
    let dst = paths::lists_dir().join("ipset-all.txt");
    let backup = paths::lists_dir().join("ipset-all.txt.backup");
    if dst.exists() {
        if backup.exists() {
            std::fs::remove_file(&backup).ok();
        }
        std::fs::copy(&dst, &backup).context("backup")?;
    }
    std::fs::write(&dst, &body).context("write ipset-all.txt")?;
    Ok(body.lines().filter(|l| !l.trim().is_empty()).count())
}

pub fn fetch_remote_hosts() -> Result<String> {
    Ok(agent().get(HOSTS_URL).call()?.into_string()?)
}

pub fn read_local_hosts() -> Result<String> {
    Ok(std::fs::read_to_string(paths::hosts_file())?)
}

pub fn write_local_hosts(content: &str) -> Result<()> {
    std::fs::write(paths::hosts_file(), content)?;
    Ok(())
}

pub fn hosts_differs(remote: &str, local: &str) -> bool {
    let r_first = remote.lines().next().unwrap_or("").trim();
    let r_last = remote.lines().filter(|l| !l.trim().is_empty()).last().unwrap_or("").trim();
    !(local.contains(r_first) && local.contains(r_last))
}
