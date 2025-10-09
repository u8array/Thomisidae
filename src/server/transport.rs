use anyhow::Result;
use jsonrpc_v2::{Data, MapRouter, Server};
use reqwest::Client;
use std::io;
use std::sync::Arc;
use tokio::io::BufReader;

use super::rpc;
use super::setup::build_state;

pub fn build_server(client: &Client) -> Arc<Server<MapRouter>> {
    let state = build_state(client);

    Server::new()
        .with_data(Data::new(state))
        .with_method("initialize", rpc::initialize)
        .with_method("tools/list", rpc::tools_list)
        .with_method("tools/call", rpc::tools_call)
        .finish()
}

pub async fn run_with_server(server: Arc<Server<jsonrpc_v2::MapRouter>>) -> Result<()> {
    let stdin = BufReader::new(tokio::io::stdin());
    let stdout = io::stdout();
    crate::server::io::run_with_io(server, stdin, stdout).await
}