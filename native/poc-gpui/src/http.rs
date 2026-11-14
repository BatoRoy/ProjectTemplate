//! A tiny blocking HTTP/1.1 GET for the backend's JSON endpoints (plain
//! `http://` only, like the Electron client's `net:request` against a local or
//! LAN backend). Runs on GPUI's background executor.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

impl Response {
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

fn split_url(url: &str) -> Result<(String, u16, String), String> {
    let rest = url.strip_prefix("http://").ok_or("only http:// URLs are supported")?;
    let (hostport, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let (host, port) = match hostport.rsplit_once(':') {
        Some((h, p)) if !h.ends_with(']') || h.starts_with('[') => {
            (h.trim_matches(|c| c == '[' || c == ']').to_string(), p.parse().map_err(|_| "bad port")?)
        }
        _ => (hostport.to_string(), 80),
    };
    if host.is_empty() {
        return Err("missing host".into());
    }
    Ok((host, port, path.to_string()))
}

/// GET `url`. Electron's `net:request` has no timeout (spec §9.9); we use 5 s
/// so "Test connection" against an unroutable address cannot hang for minutes.
pub fn get(url: &str) -> Result<Response, String> {
    let (host, port, path) = split_url(url)?;
    let timeout = Duration::from_secs(5);
    let addr = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("no address")?;
    let mut s = TcpStream::connect_timeout(&addr, timeout).map_err(|e| e.to_string())?;
    s.set_read_timeout(Some(timeout)).ok();
    s.set_write_timeout(Some(timeout)).ok();
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nAccept: application/json\r\nConnection: close\r\nUser-Agent: batopoc-gpui\r\n\r\n"
    );
    s.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&buf);
    let (head, body) = text.split_once("\r\n\r\n").ok_or("malformed response")?;
    let status = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|c| c.parse().ok())
        .ok_or("malformed status line")?;
    let chunked = head.lines().any(|l| {
        let l = l.to_ascii_lowercase();
        l.starts_with("transfer-encoding:") && l.contains("chunked")
    });
    let body = if chunked { dechunk(body) } else { body.to_string() };
    Ok(Response { status, body })
}

fn dechunk(mut s: &str) -> String {
    let mut out = String::new();
    while let Some((size, rest)) = s.split_once("\r\n") {
        let n = usize::from_str_radix(size.trim(), 16).unwrap_or(0);
        if n == 0 || rest.len() < n {
            break;
        }
        out.push_str(&rest[..n]);
        s = rest[n..].trim_start_matches("\r\n");
    }
    out
}

/// `GET {base}/api/health` succeeded.
pub fn health(base: &str) -> bool {
    get(&format!("{base}/api/health")).map(|r| r.ok()).unwrap_or(false)
}

/// `GET {base}/api/info` → version.
pub fn info_version(base: &str) -> Option<String> {
    let r = get(&format!("{base}/api/info")).ok()?;
    if !r.ok() {
        return None;
    }
    let v: serde_json::Value = serde_json::from_str(&r.body).ok()?;
    v.get("version")?.as_str().map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_urls() {
        assert_eq!(split_url("http://127.0.0.1:42997").unwrap(), ("127.0.0.1".into(), 42997, "/".into()));
        assert_eq!(split_url("http://localhost:8080/api/health").unwrap(), ("localhost".into(), 8080, "/api/health".into()));
        assert_eq!(split_url("http://example.com/x").unwrap(), ("example.com".into(), 80, "/x".into()));
        assert!(split_url("https://x").is_err());
    }

    #[test]
    fn dechunks() {
        assert_eq!(dechunk("4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n"), "Wikipedia");
    }
}
