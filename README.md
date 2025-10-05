# lm_mcp_server

This small MCP (Model Context Protocol) STDIO server binary is intended to provide controlled web access to LM Studio models

In short: instead of giving a model unrestricted internet access, this server exposes a small set of well-defined, safer tools over MCP/STDIO that fetch HTTP content or extract links. That lets the host application (for example LM Studio) retain control over requests, filtering and auditing.

Exposed tools:

- `fetch_url_text` — fetches the HTML body content of a URL and returns it as plain text (with optional truncation).
- `fetch_page_links` — extracts all href links from a page and returns them as JSON text.

Implementation note: this project uses the `mcp-protocol-sdk` Rust crate and implements MCP over STDIO.

## Why this tool?

LM Studio can launch and call external tools over MCP. This repository provides a small, auditable bridge that allows models to retrieve web content without giving the model direct network access. This enables:

- explicit input validation (allow only specific hosts/protocols),
- rate limiting and timeouts,
- content filtering (e.g. extract only text, avoid executing scripts),
- logging/auditing of requests.

## Build

Requires Rust and cargo.

```powershell
cargo build --release
```

## Run (server)

Start the server — it will read MCP/JSON-RPC requests from stdin and write responses to stdout:

```powershell
cargo run --release
```

The server implements the MCP initialization and the `tools/list` and `tools/call` methods.

## LM Studio integration (short)

1. Build the binary (see Build).
2. In LM Studio: create a new external tool (MCP) and set the command to start the compiled EXE (for example `C:\path\to\lm_mcp_server.exe`). LM Studio will launch the binary and communicate over STDIO.
3. LM Studio will perform the MCP handshake and call `tools/list`. After that it can send `tools/call` requests for available tools.

Example: `tools/list` (to check available tools)

```json
{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}
```

Example: `tools/call` (fetch links)

```json
{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
        "name": "fetch_page_links",
        "arguments": { "url": "https://example.com" }
    }
}
```

Example response:

```json
{
    "jsonrpc": "2.0",
    "id": 2,
    "result": {
        "links": ["https://example.com/about", "https://example.com/contact"]
    }
}
```

## Tool contract

Short description of tools (Inputs / Outputs / Errors):

- fetch_url_text
  - Input: { "url": string, optional: { "max_chars": number } }
  - Output: { "text": string }
  - Errors: HTTP error, timeout, invalid-url -> return an MCP error response

- fetch_page_links
  - Input: { "url": string }
  - Output: { "links": [string] }
  - Errors: same as above

Edge cases:

- empty/missing URL
- invalid scheme (e.g. ftp://, file://)
- very large responses
- redirect loops or too many redirects
