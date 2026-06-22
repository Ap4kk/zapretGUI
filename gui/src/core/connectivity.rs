use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeOutcome {
    Ok,
    Timeout,
    Refused,
    DnsFail,
    TlsFail,
    Other,
}

impl ProbeOutcome {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Timeout => "timeout",
            Self::Refused => "refused",
            Self::DnsFail => "dns fail",
            Self::TlsFail => "tls fail",
            Self::Other => "error",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub target: String,
    pub host: String,
    pub port: u16,
    pub outcome: ProbeOutcome,
    pub resolved: Option<SocketAddr>,
    pub dns_ms: Option<u128>,
    pub tcp_ms: Option<u128>,
    pub tls_ms: Option<u128>,
    pub http_ms: Option<u128>,
    pub detail: String,
}

impl ProbeResult {
    pub fn total_ms(&self) -> Option<u128> {
        let mut total: u128 = 0;
        let mut any = false;
        for v in [self.dns_ms, self.tcp_ms, self.tls_ms, self.http_ms] {
            if let Some(v) = v {
                total = total.saturating_add(v);
                any = true;
            }
        }
        if any { Some(total) } else { None }
    }
}

/// Catalog of well-known endpoints to test.
pub struct Probe {
    pub target_label: &'static str,
    pub host: &'static str,
    pub port: u16,
    pub https_path: Option<&'static str>,
}

pub const TARGETS: &[Probe] = &[
    Probe { target_label: "Discord (web)",   host: "discord.com",          port: 443, https_path: Some("/api/v9/gateway") },
    Probe { target_label: "Discord (CDN)",   host: "cdn.discordapp.com",   port: 443, https_path: Some("/") },
    Probe { target_label: "Discord (media)", host: "media.discordapp.net", port: 443, https_path: Some("/") },
    Probe { target_label: "Discord gateway", host: "gateway.discord.gg",   port: 443, https_path: None },
    Probe { target_label: "YouTube",         host: "www.youtube.com",      port: 443, https_path: Some("/") },
    Probe { target_label: "Google video",    host: "googlevideo.com",      port: 443, https_path: None },
    Probe { target_label: "YT image CDN",    host: "i.ytimg.com",          port: 443, https_path: Some("/") },
    Probe { target_label: "Voice (UDP test)",host: "discord.com",          port: 80,  https_path: None },
];

/// Runs a TCP-only probe (fast).
pub fn tcp_probe(host: &str, port: u16, timeout: Duration) -> ProbeResult {
    let mut r = ProbeResult {
        target: format!("{}:{}", host, port),
        host: host.to_string(),
        port,
        outcome: ProbeOutcome::Other,
        resolved: None,
        dns_ms: None,
        tcp_ms: None,
        tls_ms: None,
        http_ms: None,
        detail: String::new(),
    };

    let t0 = Instant::now();
    let mut addrs = match (host, port).to_socket_addrs() {
        Ok(it) => it,
        Err(e) => {
            r.outcome = ProbeOutcome::DnsFail;
            r.detail = e.to_string();
            return r;
        }
    };
    r.dns_ms = Some(t0.elapsed().as_millis());

    let addr = match addrs.next() {
        Some(a) => a,
        None => {
            r.outcome = ProbeOutcome::DnsFail;
            r.detail = "no addresses returned".into();
            return r;
        }
    };
    r.resolved = Some(addr);

    let t1 = Instant::now();
    match TcpStream::connect_timeout(&addr, timeout) {
        Ok(stream) => {
            r.tcp_ms = Some(t1.elapsed().as_millis());
            let _ = stream.set_read_timeout(Some(timeout));
            r.outcome = ProbeOutcome::Ok;
        }
        Err(e) => {
            r.detail = e.to_string();
            r.outcome = match e.kind() {
                std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => ProbeOutcome::Timeout,
                std::io::ErrorKind::ConnectionRefused => ProbeOutcome::Refused,
                _ => ProbeOutcome::Other,
            };
        }
    }
    r
}

/// Full HTTPS probe: DNS → TCP → TLS handshake (via ureq) → response status.
pub fn https_probe(probe: &Probe, timeout: Duration) -> ProbeResult {
    let mut r = tcp_probe(probe.host, probe.port, timeout);
    if r.outcome != ProbeOutcome::Ok {
        return r;
    }
    let path = match probe.https_path {
        Some(p) => p,
        None => return r,
    };
    let url = format!("https://{}{}", probe.host, path);
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .user_agent("zapret-gui-probe/1.0")
        .build();

    let t = Instant::now();
    match agent.get(&url).call() {
        Ok(resp) => {
            r.http_ms = Some(t.elapsed().as_millis());
            r.detail = format!("HTTP {}", resp.status());
            r.outcome = ProbeOutcome::Ok;
        }
        Err(ureq::Error::Status(code, _)) => {
            r.http_ms = Some(t.elapsed().as_millis());
            r.detail = format!("HTTP {}", code);
            // 2xx/3xx/4xx still means TLS+TCP worked → mark Ok
            r.outcome = ProbeOutcome::Ok;
        }
        Err(ureq::Error::Transport(t_err)) => {
            r.detail = t_err.to_string();
            let lower = r.detail.to_ascii_lowercase();
            r.outcome = if lower.contains("timed out") || lower.contains("timeout") {
                ProbeOutcome::Timeout
            } else if lower.contains("tls") || lower.contains("handshake") || lower.contains("certificate") {
                ProbeOutcome::TlsFail
            } else if lower.contains("refused") {
                ProbeOutcome::Refused
            } else {
                ProbeOutcome::Other
            };
        }
    }
    r
}

/// Flush the OS DNS cache.
pub fn flush_dns() -> anyhow::Result<()> {
    let (code, out) = crate::core::winws::run_silent("ipconfig", &["/flushdns"])?;
    if code != 0 {
        anyhow::bail!("ipconfig /flushdns returned {} — {}", code, out);
    }
    Ok(())
}
