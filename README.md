# rs-mcp-url-fetcher

[![Rust CI](https://github.com/u8array/rs-mcp-url-fetcher/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/u8array/rs-mcp-url-fetcher/actions/workflows/rust.yml)
[![Dependabot](https://img.shields.io/badge/dependabot-enabled-brightgreen?logo=dependabot)](https://github.com/u8array/lm_mcp_server/security/dependabot)


This small MCP (Model Context Protocol) STDIO server binary is intended to provide controlled web access to LM Studio models

Exposed tools:

- `fetch_url_text` — fetches the HTML body content of a URL and returns it as plain text.
- `fetch_page_links` — extracts unique href links from a page and returns them as text or JSON.

Implementation note: this project uses the `mcp-protocol-sdk` Rust crate and implements MCP over STDIO.

### Tool arguments

- fetch_url_text
    - url (string, required)

- fetch_page_links
    - url (string, required)
    - same_domain (boolean, optional, default: false)
    - format (string, optional, one of: "text" | "json"; default: "text")
  
Notes for `fetch_page_links`:
- Only http/https links are returned.
- Links are normalized (fragments removed) and de-duplicated.

## Configuration

This server optionally reads a `config.toml` placed in the same directory as the executable. If no config is found, all features default to enabled.

Example `config.toml` next to the executable:

```toml
[features]
# Fetches the text content of a URL
fetch_url_text = true

# Fetches unique links from a page
fetch_page_links = true
```

If you set a feature to `false`, the tool won't be registered and won't appear in `tools/list`.

## Why this tool?

LM Studio can launch and call external tools over MCP. This repository provides a small, auditable bridge that allows models to retrieve web content without giving the model direct network access. This enables:

- Extracts readable text from HTML (scripts/styles ignored).
- Network hygiene by default: 10s request timeout and a 10-redirect cap to avoid hangs.
- Basic URL hygiene: when extracting links, only http/https URLs are returned; optional same-domain filtering.

Note: Host allowlists and request rate limiting are not implemented yet.

## LM Studio integration (short)

For detailed setup steps, see the LM Studio MCP documentation: https://lmstudio.ai/docs/app/mcp

1. Build or download the binary (see Build).
2. Configure the MCP server in LM Studio (via the Integrations dialog):

When you click the "Install" button and then choose "Edit mcp.json", LM Studio opens a dialog where you can paste or edit the integrations JSON directly.

![LM Studio: Integration dialog](docs/install.png)

Paste JSON like the following into the dialog and save it. :

```json
{
    "mcpServers": {
        "url-fetcher": {
            "command": "path/to/lm_mcp_server"
        }
    }
}
```

If you already have other tools configured in `mcp.json`, you can add this server without removing them.


3. Enable the tool in LM Studio. The application will perform the MCP handshake and call `tools/list`. Once the handshake succeeds, the available tools appear in the integrations/plugins list.

After installation you should see the tools listed as an integration/plugin:

![LM Studio: installed and initialized](docs/installed.png)

## Build

Requires the latest stable Rust toolchain.

```powershell
cargo build --release
```

## Roadmap / TODO

- [ ] Host/Domain Allowlist & Denylist
    - `config.toml` keys: `allowlist = ["example.com"]`, `denylist = ["bad.example"]`
    - Protection against internal IPs/SSRF: validate DNS resolution, check IPs against RFC1918/RFC6598/etc., prevent DNS rebinding (compare hostname → IP after connection).
- [ ] Request & Concurrency Limits
    - Global and per-host limits: `max_concurrent_requests`, `per_host_concurrency`, `requests_per_second`, `burst`
    - Implementation: semaphore for concurrency, token bucket for RPS, clear error response on limit exceeded.
- [ ] Truncation / Maximum Size for fetch_url_text
    - Options: `max_chars`, `max_bytes`, `max_response_size_bytes`
    - Behavior: optionally truncate or return an error; abort early based on headers (Content-Length) if possible.
- [ ] robots.txt Respect (configurable)
    - Config toggle: `respect_robots_txt = true|false`
    - Behavior: cache robots.txt, optionally ignore for internal tools.
- [ ] Small In-Memory Cache
    - LRU cache with TTL, max entries configurable: `cache_enabled`, `cache_ttl_seconds`, `cache_max_entries`
    - HTTP optimization: ETag / If-None-Match support to avoid unnecessary 304s.
- [ ] Advanced Text Extraction (structured output)
    - Optional format field/flag: `extract_structured = false|true`
    - Suggested structure: `{ title, top_headings: [...], excerpt, word_count, body_text }`
    - Excerpt: configurable length via `excerpt_max_chars`.
- [ ] Security & Operational Notes
    - Document default security values (e.g., timeouts, redirect limit).
    - Logging/monitoring for rejected requests, rate-limit events, and SSRF suspicion.
- [ ] Tests & Validation
    - Unit/integration tests for: allow/deny, SSRF cases (private IPs), rate limiting, cache invalidation, robots.txt parsing, truncation.
- [ ] Add Configuration Examples
    - Show minimal and security-focused examples in README/config.toml.