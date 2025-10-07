use anyhow::Result;
use lm_mcp_server::server;
use reqwest::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent("lm_mcp_server/0.1.0")
        .build()?;

    let server = server::build_server(&client);
    server::run_with_server(server).await
}
