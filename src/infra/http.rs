use std::io::Read;
use std::net::ToSocketAddrs;

use crate::domain::error::DomainError;

const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const TIMEOUT_SECS: u64 = 30;
const MAX_REDIRECTS: u32 = 5;

pub fn fetch_url(url: &str) -> Result<String, DomainError> {
    let parsed = url::Url::parse(url)
        .map_err(|e| DomainError::InvalidInput(format!("invalid URL: {}", e)))?;

    validate_url(&parsed)?;

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(TIMEOUT_SECS))
        .timeout_read(std::time::Duration::from_secs(TIMEOUT_SECS))
        .redirects(0)
        .build();

    let mut current_url = url.to_string();
    let mut redirects = 0u32;

    loop {
        let response = agent
            .get(&current_url)
            .call()
            .map_err(|e| match e {
                ureq::Error::Status(status, resp) => {
                    if (301..=308).contains(&status) {
                        if let Some(location) = resp.header("Location") {
                            return DomainError::Network(format!("redirect:{}", location));
                        }
                    }
                    DomainError::Network(format!("HTTP {} error", status))
                }
                other => DomainError::Network(format!("HTTP request failed: {}", other)),
            });

        match response {
            Ok(resp) => {
                return read_response_body(resp);
            }
            Err(DomainError::Network(msg)) if msg.starts_with("redirect:") => {
                redirects += 1;
                if redirects > MAX_REDIRECTS {
                    return Err(DomainError::Network(format!(
                        "too many redirects (limit: {})",
                        MAX_REDIRECTS
                    )));
                }

                let location = &msg["redirect:".len()..];
                let redirect_url = resolve_redirect(&current_url, location)?;
                let redirect_parsed = url::Url::parse(&redirect_url)
                    .map_err(|e| DomainError::InvalidInput(format!("invalid redirect URL: {}", e)))?;

                validate_url(&redirect_parsed)?;
                current_url = redirect_url;
            }
            Err(e) => return Err(e),
        }
    }
}

fn resolve_redirect(base: &str, location: &str) -> Result<String, DomainError> {
    let base_url = url::Url::parse(base)
        .map_err(|e| DomainError::InvalidInput(format!("invalid base URL: {}", e)))?;
    let resolved = base_url
        .join(location)
        .map_err(|e| DomainError::InvalidInput(format!("invalid redirect location: {}", e)))?;
    Ok(resolved.to_string())
}

fn validate_url(parsed: &url::Url) -> Result<(), DomainError> {
    match parsed.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(DomainError::InvalidInput(format!(
                "unsupported URL scheme: {} (only http/https allowed)",
                scheme
            )));
        }
    }
    validate_host(parsed)
}

fn read_response_body(response: ureq::Response) -> Result<String, DomainError> {
    let content_length = response
        .header("Content-Length")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);

    if content_length > MAX_RESPONSE_SIZE {
        return Err(DomainError::InvalidInput(format!(
            "response too large: {} bytes (limit: {} bytes)",
            content_length, MAX_RESPONSE_SIZE
        )));
    }

    let mut body = Vec::new();
    let mut reader = response.into_reader();
    let mut buf = [0u8; 8192];
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| DomainError::Network(format!("failed to read response: {}", e)))?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&buf[..n]);
        if body.len() > MAX_RESPONSE_SIZE {
            return Err(DomainError::InvalidInput(format!(
                "response exceeded {} byte limit",
                MAX_RESPONSE_SIZE
            )));
        }
    }

    String::from_utf8(body)
        .map_err(|e| DomainError::Parse(format!("response is not valid UTF-8: {}", e)))
}

pub fn html_to_markdown(html: &str) -> String {
    htmd::convert(html).unwrap_or_else(|_| html.to_string())
}

fn validate_host(parsed: &url::Url) -> Result<(), DomainError> {
    let host = parsed
        .host_str()
        .ok_or_else(|| DomainError::InvalidInput("URL has no host".to_string()))?;

    let blocked_hosts = [
        "localhost",
        "metadata.google.internal",
        "metadata.google",
    ];

    let host_lower = host.to_lowercase();
    for blocked in &blocked_hosts {
        if host_lower == *blocked {
            return Err(DomainError::InvalidInput(format!(
                "access to {} is not allowed",
                host
            )));
        }
    }

    // Check IP literals directly
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        if is_blocked_ip(&ip) {
            return Err(DomainError::InvalidInput(format!(
                "access to {} is not allowed",
                host
            )));
        }
    }

    // DNS resolve and check all resolved IPs
    let port = parsed.port().unwrap_or(match parsed.scheme() {
        "https" => 443,
        _ => 80,
    });
    let addr = format!("{}:{}", host, port);
    if let Ok(addrs) = addr.to_socket_addrs() {
        for socket_addr in addrs {
            if is_blocked_ip(&socket_addr.ip()) {
                return Err(DomainError::InvalidInput(format!(
                    "access to {} is not allowed (resolves to blocked IP {})",
                    host,
                    socket_addr.ip()
                )));
            }
        }
    }

    Ok(())
}

fn is_blocked_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => is_blocked_ipv4(v4),
        std::net::IpAddr::V6(v6) => {
            if v6.is_loopback() || v6.is_unspecified() {
                return true;
            }
            // IPv6 link-local (fe80::/10)
            if (v6.segments()[0] & 0xffc0) == 0xfe80 {
                return true;
            }
            // IPv4-mapped IPv6 (::ffff:x.x.x.x) - check the embedded IPv4
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_blocked_ipv4(&v4);
            }
            false
        }
    }
}

fn is_blocked_ipv4(v4: &std::net::Ipv4Addr) -> bool {
    v4.is_loopback()             // 127.0.0.0/8
        || v4.is_private()       // 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
        || v4.is_link_local()    // 169.254.0.0/16
        || v4.is_unspecified()   // 0.0.0.0
        || v4.is_broadcast()     // 255.255.255.255
}
