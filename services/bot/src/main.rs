use bot::config::Config;
use bot::error::BotError;
use bot::repository::create_pool;
use bot::services::core_ws::start_realtime_ws_service;
use bot::services::presence::spawn_presence_loop;
use bot::startup::build_bot_client;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), BotError> {
    dotenv().ok();

    println!("Starting Bot...");

    let config = Config::from_env()
        .map_err(|e| BotError::Config(format!("Failed to load config: {}", e)))?;

    let db = create_pool(&config.db_path)
        .await
        .map_err(|e| BotError::Config(format!("Failed to initialize database: {}", e)))?;

    if let Err(e) = bot::services::price_alert::load_alerts_to_cache(&db).await {
        println!("[WARN] Failed to load price alerts to cache: {}", e);
    }

    let bot_client = build_bot_client(&config, db.clone()).await?;
    let mut client = bot_client.client;

    spawn_presence_loop(bot_client.shard_manager);

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    start_realtime_ws_service(
        db,
        bot_client.http,
        config.realtime_ws_url.clone(),
        config.client_id.clone(),
    );
    println!(
        "[OK] Realtime WebSocket service started (connecting to {})",
        config.realtime_ws_url
    );

    client
        .start()
        .await
        .map_err(|e| BotError::Client(format!("Failed to initialize client: {}", e)))?;

    Ok(())
}
