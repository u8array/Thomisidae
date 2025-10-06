
use anyhow::Result;
use lm_mcp_server::server;

#[tokio::main]
async fn main() -> Result<()> {
    server::run().await
}
