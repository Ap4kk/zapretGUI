use crate::core::paths;
use crate::core::service::{is_process_running, query_service, ServiceState};
use crate::core::winws::run_silent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Info,
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
}

pub fn run_all() -> Vec<CheckResult> {
    let mut out = Vec::new();
    out.push(check_bfe());
    out.push(check_proxy());
    out.push(check_tcp_timestamps());
    out.push(check_process_named("AdguardSvc.exe", "Adguard may cause problems with Discord"));
    out.push(check_service_substr("Killer", "Killer conflicts with zapret"));
    out.push(check_service_substr_multi(
        &["Intel", "Connectivity", "Network"],
        "Intel Connectivity Network Service conflicts with zapret",
    ));
    out.push(check_checkpoint());
    out.push(check_service_substr("SmartByte", "SmartByte conflicts with zapret"));
    out.push(check_windivert_file());
    out.push(check_vpn());
    out.push(check_secure_dns());
    out.push(check_hosts_youtube());
    out.push(check_windivert_conflict());
    out.push(check_conflicting_bypasses());
    out
}

fn check_bfe() -> CheckResult {
    match query_service("BFE") {
        ServiceState::Running => CheckResult {
            name: "Base Filtering Engine".into(),
            status: CheckStatus::Pass,
            message: "Service is running".into(),
        },
        s => CheckResult {
            name: "Base Filtering Engine".into(),
            status: CheckStatus::Fail,
            message: format!("BFE is required for zapret to work (state: {})", s.label()),
        },
    }
}

fn check_proxy() -> CheckResult {
    let (_, out) = match run_silent(
        "reg",
        &[
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
            "/v",
            "ProxyEnable",
        ],
    ) {
        Ok(v) => v,
        Err(_) => {
            return CheckResult {
                name: "System proxy".into(),
                status: CheckStatus::Info,
                message: "Could not query proxy settings".into(),
            }
        }
    };
    if out.to_ascii_lowercase().contains("0x1") {
        let server = run_silent(
            "reg",
            &[
                "query",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
                "/v",
                "ProxyServer",
            ],
        )
        .map(|(_, s)| s)
        .unwrap_or_default();
        CheckResult {
            name: "System proxy".into(),
            status: CheckStatus::Warn,
            message: format!(
                "System proxy is enabled. Disable it if you don't actually use a proxy.\n{}",
                server.trim()
            ),
        }
    } else {
        CheckResult {
            name: "System proxy".into(),
            status: CheckStatus::Pass,
            message: "Proxy is not enabled".into(),
        }
    }
}

fn check_tcp_timestamps() -> CheckResult {
    let (_, out) = match run_silent("netsh", &["interface", "tcp", "show", "global"]) {
        Ok(v) => v,
        Err(_) => {
            return CheckResult {
                name: "TCP timestamps".into(),
                status: CheckStatus::Info,
                message: "netsh failed".into(),
            }
        }
    };
    let enabled = out
        .lines()
        .find(|l| l.to_ascii_lowercase().contains("timestamps"))
        .map(|l| l.to_ascii_lowercase().contains("enabled"))
        .unwrap_or(false);

    if enabled {
        CheckResult {
            name: "TCP timestamps".into(),
            status: CheckStatus::Pass,
            message: "Enabled".into(),
        }
    } else {
        CheckResult {
            name: "TCP timestamps".into(),
            status: CheckStatus::Warn,
            message: "Disabled. Click \"Fix\" to enable.".into(),
        }
    }
}

pub fn fix_tcp_timestamps() -> anyhow::Result<()> {
    let (code, out) = run_silent("netsh", &["interface", "tcp", "set", "global", "timestamps=enabled"])?;
    if code != 0 {
        anyhow::bail!("netsh failed (code {}): {}", code, out);
    }
    Ok(())
}

fn check_process_named(image: &str, msg: &str) -> CheckResult {
    if is_process_running(image) {
        CheckResult {
            name: image.to_string(),
            status: CheckStatus::Warn,
            message: msg.to_string(),
        }
    } else {
        CheckResult {
            name: image.to_string(),
            status: CheckStatus::Pass,
            message: "Process not found".into(),
        }
    }
}

fn check_service_substr(needle: &str, msg: &str) -> CheckResult {
    let (_, out) = run_silent("sc", &["query"]).unwrap_or((0, String::new()));
    if out.to_ascii_lowercase().contains(&needle.to_ascii_lowercase()) {
        CheckResult {
            name: needle.to_string(),
            status: CheckStatus::Warn,
            message: msg.to_string(),
        }
    } else {
        CheckResult {
            name: needle.to_string(),
            status: CheckStatus::Pass,
            message: "Not found".into(),
        }
    }
}

fn check_service_substr_multi(needles: &[&str], msg: &str) -> CheckResult {
    let (_, out) = run_silent("sc", &["query"]).unwrap_or((0, String::new()));
    let lower = out.to_ascii_lowercase();
    let found = needles.iter().all(|n| lower.contains(&n.to_ascii_lowercase()));
    let label = needles.join(" ");
    if found {
        CheckResult { name: label, status: CheckStatus::Warn, message: msg.to_string() }
    } else {
        CheckResult { name: label, status: CheckStatus::Pass, message: "Not found".into() }
    }
}

fn check_checkpoint() -> CheckResult {
    let (_, out) = run_silent("sc", &["query"]).unwrap_or((0, String::new()));
    let lower = out.to_ascii_lowercase();
    let found = lower.contains("tracsrvwrapper") || lower.contains("epwd");
    if found {
        CheckResult {
            name: "Check Point".into(),
            status: CheckStatus::Warn,
            message: "Check Point services found. They conflict with zapret".into(),
        }
    } else {
        CheckResult {
            name: "Check Point".into(),
            status: CheckStatus::Pass,
            message: "Not found".into(),
        }
    }
}

fn check_windivert_file() -> CheckResult {
    let bin = paths::bin_dir();
    let ok = std::fs::read_dir(&bin)
        .map(|rd| {
            rd.flatten()
                .any(|e| e.path().extension().map(|ext| ext.eq_ignore_ascii_case("sys")).unwrap_or(false))
        })
        .unwrap_or(false);
    if ok {
        CheckResult {
            name: "WinDivert64.sys".into(),
            status: CheckStatus::Pass,
            message: "Driver file present in bin/".into(),
        }
    } else {
        CheckResult {
            name: "WinDivert64.sys".into(),
            status: CheckStatus::Fail,
            message: "Driver file is missing from bin/".into(),
        }
    }
}

fn check_vpn() -> CheckResult {
    let (_, out) = run_silent("sc", &["query"]).unwrap_or((0, String::new()));
    let vpns: Vec<String> = out
        .lines()
        .filter_map(|l| {
            let lt = l.trim();
            if lt.to_ascii_lowercase().contains("vpn") && lt.starts_with("SERVICE_NAME") {
                Some(lt.replace("SERVICE_NAME:", "").trim().to_string())
            } else {
                None
            }
        })
        .collect();
    if vpns.is_empty() {
        CheckResult {
            name: "VPN".into(),
            status: CheckStatus::Pass,
            message: "No VPN services found".into(),
        }
    } else {
        CheckResult {
            name: "VPN".into(),
            status: CheckStatus::Warn,
            message: format!("VPN services found: {}. Some VPNs conflict with zapret.", vpns.join(", ")),
        }
    }
}

fn check_secure_dns() -> CheckResult {
    let script = "Get-ChildItem -Recurse -Path 'HKLM:System\\CurrentControlSet\\Services\\Dnscache\\InterfaceSpecificParameters\\' | Get-ItemProperty | Where-Object { $_.DohFlags -gt 0 } | Measure-Object | Select-Object -ExpandProperty Count";
    let (_, out) = run_silent("powershell", &["-NoProfile", "-Command", script]).unwrap_or((0, String::new()));
    let count: u32 = out.trim().lines().last().unwrap_or("0").trim().parse().unwrap_or(0);
    if count > 0 {
        CheckResult {
            name: "Secure DNS".into(),
            status: CheckStatus::Pass,
            message: format!("{} encrypted DNS endpoints configured", count),
        }
    } else {
        CheckResult {
            name: "Secure DNS".into(),
            status: CheckStatus::Warn,
            message: "Configure Secure DNS in your browser or in Windows 11 settings".into(),
        }
    }
}

fn check_hosts_youtube() -> CheckResult {
    let path = paths::hosts_file();
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let lower = content.to_ascii_lowercase();
    if lower.contains("youtube.com") || lower.contains("youtu.be") {
        CheckResult {
            name: "hosts file".into(),
            status: CheckStatus::Warn,
            message: "Contains entries for youtube.com / youtu.be — may break YouTube".into(),
        }
    } else {
        CheckResult {
            name: "hosts file".into(),
            status: CheckStatus::Pass,
            message: "No YouTube overrides".into(),
        }
    }
}

fn check_windivert_conflict() -> CheckResult {
    let winws = is_process_running("winws.exe");
    let wd = matches!(query_service("WinDivert"), ServiceState::Running | ServiceState::StopPending | ServiceState::StartPending);
    if !winws && wd {
        CheckResult {
            name: "WinDivert conflict".into(),
            status: CheckStatus::Warn,
            message: "winws.exe is not running but WinDivert service is active. Another bypass may be using it.".into(),
        }
    } else {
        CheckResult {
            name: "WinDivert conflict".into(),
            status: CheckStatus::Pass,
            message: "No conflicting state detected".into(),
        }
    }
}

fn check_conflicting_bypasses() -> CheckResult {
    let candidates = ["GoodbyeDPI", "discordfix_zapret", "winws1", "winws2"];
    let mut found = Vec::new();
    for c in candidates {
        if !matches!(query_service(c), ServiceState::NotInstalled) {
            found.push(c);
        }
    }
    if found.is_empty() {
        CheckResult {
            name: "Conflicting bypasses".into(),
            status: CheckStatus::Pass,
            message: "None found".into(),
        }
    } else {
        CheckResult {
            name: "Conflicting bypasses".into(),
            status: CheckStatus::Fail,
            message: format!("Found: {}. Remove them before using zapret.", found.join(", ")),
        }
    }
}
