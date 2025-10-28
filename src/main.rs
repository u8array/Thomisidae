use anyhow::Result;
use lm_mcp_server::server;
use lm_mcp_server::config::Config;
use reqwest::Client;
use std::time::Duration;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;
    let config = Config::load_default();
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent("lm_mcp_server/0.1.0")
        .build()?;

    let server = server::build_server(&client, &config);
    server::run_with_server(server).await
}
