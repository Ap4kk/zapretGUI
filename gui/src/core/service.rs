use crate::core::paths;
use crate::core::winws::run_silent;
use anyhow::{anyhow, Result};
use windows::core::PCWSTR;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ,
    REG_VALUE_TYPE,
};
use windows::Win32::System::Services::{
    CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatus, SC_HANDLE,
    SC_MANAGER_CONNECT, SERVICE_QUERY_STATUS, SERVICE_RUNNING, SERVICE_START_PENDING,
    SERVICE_STATUS, SERVICE_STOPPED, SERVICE_STOP_PENDING,
};

pub const ZAPRET_SERVICE: &str = "zapret";
pub const WINDIVERT_SERVICE: &str = "WinDivert";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceState {
    NotInstalled,
    Stopped,
    Running,
    StartPending,
    StopPending,
    Other(u32),
}

impl ServiceState {
    pub fn label(&self) -> &'static str {
        match self {
            Self::NotInstalled => "not installed",
            Self::Stopped => "stopped",
            Self::Running => "running",
            Self::StartPending => "start-pending",
            Self::StopPending => "stop-pending",
            Self::Other(_) => "unknown",
        }
    }
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn query_service(name: &str) -> ServiceState {
    unsafe {
        let scm = match OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT) {
            Ok(h) if !h.is_invalid() => h,
            _ => return ServiceState::NotInstalled,
        };
        let wname = to_wide(name);
        let svc = match OpenServiceW(scm, PCWSTR(wname.as_ptr()), SERVICE_QUERY_STATUS) {
            Ok(h) if !h.is_invalid() => h,
            _ => {
                let _ = CloseServiceHandle(scm);
                return ServiceState::NotInstalled;
            }
        };
        let mut status = SERVICE_STATUS::default();
        let result = QueryServiceStatus(svc, &mut status);
        let _ = CloseServiceHandle(svc);
        let _ = CloseServiceHandle(scm);
        if result.is_err() {
            return ServiceState::Other(0);
        }
        match status.dwCurrentState {
            SERVICE_STOPPED => ServiceState::Stopped,
            SERVICE_RUNNING => ServiceState::Running,
            SERVICE_START_PENDING => ServiceState::StartPending,
            SERVICE_STOP_PENDING => ServiceState::StopPending,
            other => ServiceState::Other(other.0),
        }
    }
}

#[allow(dead_code)]
fn _silence_unused(_h: SC_HANDLE) {}

/// Determines if a process with the given image name is running.
pub fn is_process_running(image_name: &str) -> bool {
    unsafe {
        let snap = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(h) if !h.is_invalid() => h,
            _ => return false,
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut found = false;
        if Process32FirstW(snap, &mut entry).is_ok() {
            loop {
                let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..len]);
                if name.eq_ignore_ascii_case(image_name) {
                    found = true;
                    break;
                }
                if Process32NextW(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
        found
    }
}

/// Reads the strategy file name stored by `service.bat` when installing the service.
pub fn current_installed_strategy() -> Option<String> {
    unsafe {
        let key_name = to_wide(r"System\CurrentControlSet\Services\zapret");
        let mut hkey = HKEY::default();
        let st = RegOpenKeyExW(HKEY_LOCAL_MACHINE, PCWSTR(key_name.as_ptr()), 0, KEY_READ, &mut hkey);
        if st.is_err() {
            return None;
        }
        let value_name = to_wide("zapret-discord-youtube");
        let mut ty = REG_VALUE_TYPE::default();
        let mut buf = vec![0u8; 1024];
        let mut len: u32 = buf.len() as u32;
        let r = RegQueryValueExW(
            hkey,
            PCWSTR(value_name.as_ptr()),
            None,
            Some(&mut ty),
            Some(buf.as_mut_ptr()),
            Some(&mut len),
        );
        let _ = RegCloseKey(hkey);
        if r.is_err() || ty != REG_SZ {
            return None;
        }
        let used = (len as usize).min(buf.len());
        // It's UTF-16
        let u16s: Vec<u16> = buf[..used]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .take_while(|&c| c != 0)
            .collect();
        Some(String::from_utf16_lossy(&u16s))
    }
}

/// Install zapret as a Windows service running winws.exe with the given args.
/// We shell out to `sc.exe` because building the binPath quoting via the Service Control API
/// is significantly harder, and `sc.exe` is always present.
pub fn install_service(strategy_name: &str, args: &[String]) -> Result<()> {
    let exe = paths::winws_exe();
    if !exe.exists() {
        return Err(anyhow!("winws.exe not found"));
    }

    // Stop & delete existing service first.
    let _ = run_silent("net", &["stop", ZAPRET_SERVICE]);
    let _ = run_silent("sc", &["delete", ZAPRET_SERVICE]);

    // Build the binPath: quoted exe path, then args. Quotes inside args must be escaped \"
    let mut bin_path = String::new();
    bin_path.push('"');
    bin_path.push_str(&exe.to_string_lossy());
    bin_path.push('"');
    for a in args {
        bin_path.push(' ');
        if a.contains(' ') || a.contains('\t') {
            // If arg already contains '=', split into key="value"
            if let Some(eq) = a.find('=') {
                let (k, v) = a.split_at(eq);
                let v = &v[1..]; // skip '='
                bin_path.push_str(k);
                bin_path.push('=');
                bin_path.push('\\');
                bin_path.push('"');
                bin_path.push_str(v);
                bin_path.push('\\');
                bin_path.push('"');
            } else {
                bin_path.push('\\');
                bin_path.push('"');
                bin_path.push_str(a);
                bin_path.push('\\');
                bin_path.push('"');
            }
        } else {
            bin_path.push_str(a);
        }
    }

    let bin_path_arg = format!("binPath= {}", bin_path);
    let (code, out) = run_silent(
        "sc",
        &[
            "create",
            ZAPRET_SERVICE,
            &bin_path_arg,
            "DisplayName=",
            "zapret",
            "start=",
            "auto",
        ],
    )?;
    if code != 0 {
        return Err(anyhow!("sc create failed (code {}): {}", code, out));
    }
    let _ = run_silent(
        "sc",
        &[
            "description",
            ZAPRET_SERVICE,
            "Zapret DPI bypass software",
        ],
    );
    let (code, out) = run_silent("sc", &["start", ZAPRET_SERVICE])?;
    if code != 0 {
        return Err(anyhow!("sc start failed (code {}): {}", code, out));
    }

    // Write the strategy name to the registry, mirroring service.bat behavior.
    let _ = run_silent(
        "reg",
        &[
            "add",
            r"HKLM\System\CurrentControlSet\Services\zapret",
            "/v",
            "zapret-discord-youtube",
            "/t",
            "REG_SZ",
            "/d",
            strategy_name,
            "/f",
        ],
    );

    Ok(())
}

pub fn remove_service() -> Result<()> {
    let _ = run_silent("net", &["stop", ZAPRET_SERVICE]);
    let _ = run_silent("sc", &["delete", ZAPRET_SERVICE]);
    let _ = run_silent("net", &["stop", WINDIVERT_SERVICE]);
    let _ = run_silent("sc", &["delete", WINDIVERT_SERVICE]);
    let _ = run_silent("net", &["stop", "WinDivert14"]);
    let _ = run_silent("sc", &["delete", "WinDivert14"]);
    // Kill any winws.exe that might be left over.
    let _ = run_silent("taskkill", &["/IM", "winws.exe", "/F"]);
    Ok(())
}

pub fn restart_service() -> Result<()> {
    let _ = run_silent("net", &["stop", ZAPRET_SERVICE]);
    let (code, out) = run_silent("net", &["start", ZAPRET_SERVICE])?;
    if code != 0 {
        return Err(anyhow!("net start failed (code {}): {}", code, out));
    }
    Ok(())
}
