# Thomisidae

[![Rust CI](https://github.com/u8array/thomisidae/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/u8array/thomisidae/actions/workflows/rust.yml)
[![Dependabot](https://img.shields.io/badge/dependabot-enabled-brightgreen?logo=dependabot)](https://github.com/u8array/thomisidae/security/dependabot)


This small MCP (Model Context Protocol) STDIO server binary provides controlled web access to any MCP-compatible LLM client.

Exposed tools:

- `fetch_url_text` — fetches the HTML body content of a URL and returns it as plain text.
- `fetch_page_links` — extracts unique href links from a page and returns them as text or JSON.
- `google_search` — performs a Google Programmable Search (Custom Search API) query and returns top results. (requires an API key: https://docs.cloud.google.com/docs/authentication/api-keys?hl=en#create)

## Why this tool?

Clients that implement the open Model Context Protocol (MCP) can launch and call external tools. This repository provides a small, auditable bridge that allows models to retrieve web content. LM Studio is one such MCP client and is used below as an example.

## MCP client integration (example: LM Studio)

This server works with any MCP-compatible client. The following shows setup in LM Studio as one example.

For detailed LM Studio setup steps, see the LM Studio MCP documentation: https://lmstudio.ai/docs/app/mcp

1. Build or download the binary (see Build).
2. In LM Studio, configure the MCP server (via the Integrations dialog):

When you click the "Install" button and then choose "Edit mcp.json", LM Studio opens a dialog where you can paste or edit the integrations JSON directly.

![LM Studio: Integration dialog](docs/install.png)

Paste JSON like the following into the dialog and save it:

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


3. In LM Studio, enable the tool. The application will perform the MCP handshake and call `tools/list`. Once the handshake succeeds, the available tools appear in the integrations/plugins list.

After installation you should see the tools listed as an integration/plugin:

![LM Studio: installed and initialized](docs/installed.png)

## Tool arguments

- fetch_url_text
    - url (string, required)

- fetch_page_links
    - url (string, required)
    - same_domain (boolean, optional, default: false)
    - format (string, optional, one of: "text" | "json"; default: "text")

    Notes for `fetch_page_links`:
    - Only http/https links are returned.
    - Links are normalized (fragments removed) and de-duplicated.

- google_search
    - query (string, required)
    - num (integer, optional, 1-10; default: 5)
    - site (string, optional; restricts to a domain like "example.com")
    - format (string, optional, one of: "text" | "json"; default: "text")
 
  Notes for `google_search`:
    - Requires either config keys `google_search.api_key` and `google_search.cse_id` in `config.toml`, or environment variables `GOOGLE_API_KEY` and `GOOGLE_CSE_ID`.
    - Uses Google Custom Search JSON API. You need to create a Programmable Search Engine (CSE) and enable the Custom Search API in Google Cloud.
  

## Configuration

This server optionally reads a `config.toml` placed in the same directory as the executable. If no config is found, all features default to enabled.
Environment variables can also be loaded from a local `.env` file (dotenv) automatically at startup. This is handy for secrets like `GOOGLE_API_KEY`.

Example `config.toml` next to the executable:

```toml
[features]
# Fetches the text content of a URL
fetch_url_text = true

# Fetches unique links from a page
fetch_page_links = true

# Enable Google Custom Search tool
google_search = true

# Google Programmable Search configuration (optional; can also use env vars)
[google_search]
api_key = "YOUR_GOOGLE_API_KEY"
cse_id = "YOUR_CUSTOM_SEARCH_ENGINE_ID"

# Robots.txt compliance
[robots]
# Respect robots.txt rules when fetching pages
obey = true
# Optional UA used both for robots evaluation and HTTP requests (if provided)
# user_agent = "lm_mcp_server/0.1.0"
# Cache TTL for per-origin robots rules
cache_ttl_secs = 3600
```

If you set a feature to `false`, the tool won't be registered and won't appear in `tools/list`.


Example `.env`:

```
GOOGLE_API_KEY=your_api_key_here
GOOGLE_CSE_ID=your_cse_id_here
```

## robots.txt handling

- The server enforces robots.txt for page fetches (`fetch_url_text`, `fetch_page_links`) when `robots.obey = true` (default).
- Per origin, `robots.txt` is fetched and cached for `robots.cache_ttl_secs` seconds.
- Parsing and matching use the `robotstxt` crate (a native Rust port of Google’s robots.txt parser and matcher), so semantics align closely with industry expectations.
- If `robots.txt` can’t be fetched (non-success HTTP) or the request fails, we default to allow (fail-open). Disable entirely via `robots.obey = false`.

## Build

Requires the latest stable Rust toolchain.

```powershell
cargo build --release
```

