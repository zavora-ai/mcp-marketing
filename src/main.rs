mod client;
mod server;

use client::detect_backend;
use rmcp::{ServiceExt, transport::stdio};
use server::MarketingServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    let api = detect_backend()?;
    let service = MarketingServer { api }.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
