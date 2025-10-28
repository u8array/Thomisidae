use mcp_protocol_sdk::prelude::*;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use url::Url;
use futures_util::StreamExt;


pub fn required_str_arg(
    arguments: &HashMap<String, Value>,
    key: &str,
) -> McpResult<String> {
    arguments
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| McpError::validation(format!("Missing '{key}' parameter")))
}


pub async fn fetch_html(client: &Client, url: &str, max_response_size: usize) -> McpResult<String> {
    let parsed = Url::parse(url).map_err(|e| McpError::validation(format!("Invalid url: {e}")))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(McpError::validation(format!(
            "Unsupported URL scheme: {} (only http/https allowed)", parsed.scheme()
        )));
    }
    
    if let Some(host) = parsed.host_str()
        && let Ok(ip) = host.parse::<IpAddr>()
        && !is_global_ip(ip)
    {
        return Err(McpError::validation("URL host resolves to a non-global IP (blocked)".to_string()));
    }

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| McpError::internal(e.to_string()))?;


    if let Some(len) = resp.content_length()
        && (len as usize > max_response_size)
    {
        return Err(McpError::validation(format!(
            "Response too large: {len} bytes (max {max_response_size})"
        )));
    }


    let mut total: usize = 0;
    let mut out = Vec::with_capacity(64 * 1024);
    let mut stream = resp.bytes_stream();
    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res.map_err(|e| McpError::internal(e.to_string()))?;
        total = total.saturating_add(chunk.len());
        if total > max_response_size {
            return Err(McpError::validation(format!(
                "Response exceeded limit ({max_response_size} bytes)"
            )));
        }
        out.extend_from_slice(&chunk);
    }

    let text = String::from_utf8_lossy(&out).into_owned();
    Ok(text)
}

fn is_global_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_global_ipv4(v4),
        IpAddr::V6(v6) => is_global_ipv6(v6),
    }
}

fn is_global_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    // 10.0.0.0/8
    if octets[0] == 10 { return false; }
    // 172.16.0.0/12
    if octets[0] == 172 && (16..=31).contains(&octets[1]) { return false; }
    // 192.168.0.0/16
    if octets[0] == 192 && octets[1] == 168 { return false; }
    // 127.0.0.0/8 loopback
    if octets[0] == 127 { return false; }
    // 169.254.0.0/16 link-local
    if octets[0] == 169 && octets[1] == 254 { return false; }
    // 0.0.0.0/8, 255.255.255.255 broadcast
    if octets[0] == 0 || ip == Ipv4Addr::new(255,255,255,255) { return false; }
    // 224.0.0.0/4 multicast & 240.0.0.0/4 reserved
    if (224..=255).contains(&octets[0]) { return false; }
    true
}

fn is_global_ipv6(ip: Ipv6Addr) -> bool {
    let seg0 = ip.segments()[0];
    // ::/128 unspecified
    if ip.is_unspecified() { return false; }
    // ::1/128 loopback
    if ip.is_loopback() { return false; }
    // fe80::/10 link-local
    if (seg0 & 0xffc0) == 0xfe80 { return false; }
    // fc00::/7 unique local
    if (seg0 & 0xfe00) == 0xfc00 { return false; }
    // ff00::/8 multicast
    if (seg0 & 0xff00) == 0xff00 { return false; }
    true
}


pub fn text_tool_result<T: Into<String>>(text: T) -> ToolResult {
    ToolResult {
        content: vec![Content::Text {
            text: text.into(),
            annotations: None,
            meta: None,
        }],
        is_error: Some(false),
        structured_content: None,
        meta: None,
    }
}
