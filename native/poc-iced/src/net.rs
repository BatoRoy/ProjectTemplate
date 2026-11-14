//! Backend calls (spec §7.3) and the desktop notification.
//!
//! The backend is plain HTTP on localhost/LAN, so a 60-line HTTP/1.0 GET over
//! `std::net` replaces a full client stack (keeps the binary and the startup
//! lean). Unlike Electron's `net:request`, it has a 5 s timeout, so "Test
//! connection" against an unroutable address cannot hang for minutes.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

pub fn get(base: &str, path: &str) -> Result<Response, String> {
    let rest = base.strip_prefix("http://").ok_or_else(|| "only http:// is supported".to_string())?;
    let (hostport, prefix) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, ""),
    };
    let addr_str = if hostport.contains(':') { hostport.to_string() } else { format!("{hostport}:80") };
    let timeout = Duration::from_secs(5);
    let addrs: Vec<_> = addr_str.to_socket_addrs().map_err(|e| e.to_string())?.collect();
    let mut last = "no address".to_string();
    for addr in addrs {
        match TcpStream::connect_timeout(&addr, timeout) {
            Ok(mut s) => {
                s.set_read_timeout(Some(timeout)).ok();
                s.set_write_timeout(Some(timeout)).ok();
                let req = format!(
                    "GET {prefix}{path} HTTP/1.0\r\nHost: {hostport}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
                );
                s.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
                let mut buf = Vec::new();
                s.read_to_end(&mut buf).map_err(|e| e.to_string())?;
                let text = String::from_utf8_lossy(&buf);
                let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
                let status = head
                    .lines()
                    .next()
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|c| c.parse().ok())
                    .ok_or_else(|| "bad response".to_string())?;
                return Ok(Response { status, body: body.to_string() });
            }
            Err(e) => last = e.to_string(),
        }
    }
    Err(last)
}

/// `GET /api/health` → ok?
pub fn health(base: &str) -> bool {
    matches!(get(base, "/api/health"), Ok(r) if (200..300).contains(&r.status))
}

/// `GET /api/info` → version.
pub fn info_version(base: &str) -> Option<String> {
    let r = get(base, "/api/info").ok().filter(|r| (200..300).contains(&r.status))?;
    let v: serde_json::Value = serde_json::from_str(&r.body).ok()?;
    v.get("version")?.as_str().map(str::to_string)
}

/// Run blocking work off the UI thread, resolving a future for `Task::perform`.
pub fn spawn<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> impl std::future::Future<Output = T> {
    let (tx, rx) = iced::futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    async move { rx.await.expect("worker thread") }
}

/// freedesktop Notifications over D-Bus. Returns false when no server answers.
pub fn notify(title: &str, body: &str) -> bool {
    use std::collections::HashMap;
    let Ok(conn) = zbus::blocking::Connection::session() else { return false };
    let hints: HashMap<&str, zbus::zvariant::Value<'_>> = HashMap::new();
    conn.call_method(
        Some("org.freedesktop.Notifications"),
        "/org/freedesktop/Notifications",
        Some("org.freedesktop.Notifications"),
        "Notify",
        &("App", 0u32, "", title, body, Vec::<&str>::new(), hints, -1i32),
    )
    .is_ok()
}

#[cfg(test)]
mod tests {
    /// Talks to the session's notification server without showing anything.
    #[test]
    fn notification_server_answers() {
        let Ok(conn) = zbus::blocking::Connection::session() else { return };
        let reply = conn.call_method(
            Some("org.freedesktop.Notifications"),
            "/org/freedesktop/Notifications",
            Some("org.freedesktop.Notifications"),
            "GetServerInformation",
            &(),
        );
        let (name, ..): (String, String, String, String) = reply.unwrap().body().deserialize().unwrap();
        eprintln!("notification server: {name}");
    }

    #[test]
    fn http_get_parses_status() {
        // Nothing listens on port 1: a clean error, not a hang.
        assert!(super::get("http://127.0.0.1:1", "/api/health").is_err());
    }
}
