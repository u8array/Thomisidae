use anyhow::Result;
use jsonrpc_v2::{Data, ResponseObjects, Server};
use reqwest::Client;
use std::io::{self, BufWriter, Write};
use tokio::io::{AsyncBufReadExt, BufReader};

use super::rpc;
use super::setup::build_state;

pub async fn run() -> Result<()> {
    let client = Client::new();
    let state = build_state(&client);

    let server = Server::new()
        .with_data(Data::new(state))
        .with_method("initialize", rpc::initialize)
        .with_method("tools/list", rpc::tools_list)
        .with_method("tools/call", rpc::tools_call)
        .finish();

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let response = server.handle(trimmed.as_bytes()).await;
        match response {
            ResponseObjects::Empty => {}
            other => {
                if let Ok(s) = serde_json::to_string(&other) {
                    let _ = writeln!(out, "{}", s);
                    let _ = out.flush();
                }
            }
        }
    }

    Ok(())
}
