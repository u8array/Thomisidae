use anyhow::Result;
use thomisidae::server;
use thomisidae::config::Config;
use thomisidae::environment;
use reqwest::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    environment::load_env();
    let config = Config::load_default();
    let ua = config
        .robots
        .user_agent
        .clone()
        .unwrap_or_else(|| "thomisidae/0.1.0".to_string());
    let client = Client::builder()
        .timeout(Duration::from_millis(config.timeout_ms))
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent(ua)
        .build()?;

    let server = server::build_server(&client, &config);
    server::run_with_server(server).await
}
