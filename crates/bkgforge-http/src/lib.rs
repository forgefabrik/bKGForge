//! Minimal real HTTP/1.1 capture transport for explicitly permitted targets.
use bkgforge_security::NetworkPolicy;
use bkgforge_trace::{Exchange, Headers, HttpMessage, HttpResponse, sanitize_headers};
use std::fmt::Write as FmtWrite;
use std::io::{Read, Write};
use std::net::TcpStream;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}
impl Method {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
        }
    }
}
#[derive(Clone, Debug)]
pub struct Request {
    pub method: Method,
    pub url: String,
    pub headers: Headers,
    pub body: Vec<u8>,
}
#[derive(Clone, Debug)]
pub struct CaptureClient {
    pub policy: NetworkPolicy,
}
impl CaptureClient {
    #[must_use]
    pub fn new(policy: NetworkPolicy) -> Self {
        Self { policy }
    }
    /// # Errors
    ///
    /// Returns an error for invalid URLs, blocked targets, connection failures, or malformed HTTP.
    pub fn execute(&self, request: Request) -> Result<Exchange, String> {
        let (host, port, path) = parse_http_url(&request.url)?;
        self.policy.check_host(host)?;
        let mut stream = TcpStream::connect((host, port)).map_err(|e| e.to_string())?;
        let mut headers = request.headers.clone();
        headers.entry("Host".into()).or_insert_with(|| host.into());
        headers
            .entry("Connection".into())
            .or_insert_with(|| "close".into());
        headers
            .entry("Content-Length".into())
            .or_insert_with(|| request.body.len().to_string());
        let mut wire = format!("{} {} HTTP/1.1\r\n", request.method.as_str(), path);
        for (k, v) in &headers {
            let _ = write!(wire, "{k}: {v}\r\n");
        }
        wire.push_str("\r\n");
        stream
            .write_all(wire.as_bytes())
            .map_err(|e| e.to_string())?;
        stream.write_all(&request.body).map_err(|e| e.to_string())?;
        let mut bytes = vec![];
        stream.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        let split = bytes
            .windows(4)
            .position(|x| x == b"\r\n\r\n")
            .ok_or("invalid HTTP response")?;
        let head = String::from_utf8_lossy(&bytes[..split]);
        let mut lines = head.lines();
        let status = lines
            .next()
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|n| n.parse().ok())
            .ok_or("invalid status")?;
        let response_headers = lines
            .filter_map(|l| {
                l.split_once(':')
                    .map(|(k, v)| (k.trim().into(), v.trim().into()))
            })
            .collect();
        let final_url = format!("http://{host}:{port}{path}");
        Ok(Exchange {
            request: HttpMessage {
                method: request.method.as_str().into(),
                url: request.url.clone(),
                headers: sanitize_headers(&headers),
                body: request.body,
            },
            response: HttpResponse {
                status,
                headers: sanitize_headers(&response_headers),
                body: bytes[split + 4..].to_vec(),
                final_url,
            },
            redirect_chain: vec![],
        })
    }
}
fn parse_http_url(url: &str) -> Result<(&str, u16, &str), String> {
    let rest = url
        .strip_prefix("http://")
        .ok_or("only http:// URLs are supported by the dependency-free capture transport")?;
    let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
    let (host, port) = authority
        .rsplit_once(':')
        .and_then(|(h, p)| p.parse().ok().map(|p| (h, p)))
        .unwrap_or((authority, 80));
    if host.is_empty() {
        return Err("URL host is empty".into());
    }
    Ok((
        host,
        port,
        if path.is_empty() {
            "/"
        } else {
            &rest[authority.len()..]
        },
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn policy_blocks_loopback_by_default() {
        assert!(
            CaptureClient::new(NetworkPolicy::default())
                .execute(Request {
                    method: Method::Get,
                    url: "http://127.0.0.1/".into(),
                    headers: Headers::new(),
                    body: vec![]
                })
                .is_err()
        );
    }
    #[test]
    fn all_methods_have_wire_names() {
        assert_eq!(Method::Options.as_str(), "OPTIONS");
        assert_eq!(Method::Patch.as_str(), "PATCH");
    }
}
