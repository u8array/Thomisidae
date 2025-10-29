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
        .http
        .user_agent
        .clone()
        .or_else(|| config.robots.user_agent.clone())
        .unwrap_or_else(|| "thomisidae/0.1.0".to_string());
    let mut builder = Client::builder()
        .timeout(Duration::from_millis(config.timeout_ms))
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent(ua);

    if let Some(proxy_url) = &config.http.proxy_url
        && !proxy_url.trim().is_empty()
    {
        builder = builder.proxy(reqwest::Proxy::all(proxy_url)?);
    }

    let client = builder.build()?;

    let server = server::build_server(&client, &config);
    server::run_with_server(server).await
}
