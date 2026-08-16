use anyhow::Result;
use scrapy::app::{App, Config};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    App::new(Config::from_env()?).run().await
}
