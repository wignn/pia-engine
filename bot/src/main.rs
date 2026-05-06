use dotenvy::dotenv;
use poise::serenity_prelude::UserId;
use serenity::all::{ActivityData, GatewayIntents, OnlineStatus};
use std::collections::HashSet;
use bot::commands::{
    Data, admin, calendar, forex, general, market, moderation, ping, stock, sys,
    twitter, volatility,
};
use bot::config::Config;
use bot::error::BotError;
use bot::handlers::{handle_event, on_error};
use bot::repository::create_pool;
use bot::services::core_ws::start_core_ws_service;

#[tokio::main]
async fn main() -> Result<(), BotError> {
    dotenv().ok();

    println!("Starting Bot...");

    let config = Config::from_env()
        .map_err(|e| BotError::Config(format!("Failed to load config: {}", e)))?;

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_MEMBERS;

    let owner_id = config.client_id.parse::<u64>()
        .expect("CLIENT_ID must be a valid u64");

    let mut owners = HashSet::new();
    owners.insert(UserId::new(owner_id));

    // SQLite — no external DB required
    let db = create_pool(&config.db_path)
        .await
        .map_err(|e| BotError::Config(format!("Failed to initialize database: {}", e)))?;

    if let Err(e) = bot::services::price_alert::load_alerts_to_cache(&db).await {
        println!("[WARN] Failed to load price alerts to cache: {}", e);
    }

    let owners_clone = owners.clone();
    let db_for_setup = db.clone();
    let db_for_ws = db.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                // General commands
                ping::ping(),
                general::ping(),
                general::say(),
                general::purge(),
                // Admin commands
                admin::everyone(),
                sys::sys(),
                moderation::warn(),
                moderation::warnings(),
                moderation::clearwarnings(),
                moderation::mute(),
                moderation::unmute(),
                moderation::kick(),
                moderation::ban(),
                moderation::unban(),
                // Auto-role commands
                moderation::autorole_set(),
                moderation::autorole_disable(),
                // Logging commands
                moderation::log_setup(),
                moderation::log_disable(),
                // Forex commands
                forex::forex_setup(),
                forex::forex_disable(),
                forex::forex_enable(),
                forex::forex_status(),
                forex::forex_calendar(),
                // Calendar reminder commands
                calendar::calendar_setup(),
                calendar::calendar_disable(),
                calendar::calendar_enable(),
                calendar::calendar_status(),
                calendar::calendar_mention(),
                // Stock news commands (subscribe/unsubscribe/status/latest via subcommands)
                stock::stocknews(),
                // Market price commands
                market::price(),
                market::prices(),
                // Price alert commands
                market::alert(),
                market::alerts(),
                market::alert_remove(),
                // Volatility spike detector commands
                volatility::volatility_setup(),
                volatility::volatility_disable(),
                volatility::volatility_status(),
                // X/Twitter feed commands
                twitter::twitter_setup(),
                twitter::twitter_disable(),
                twitter::twitter_enable(),
                twitter::twitter_status(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            on_error: |error| Box::pin(on_error(error)),
            event_handler: |ctx, event, _framework, data| Box::pin(handle_event(ctx, event, data)),
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            let inner_db = db_for_setup.clone();
            let owners_inner = owners_clone.clone();
            let core_http_inner = config.core_http_url.clone();
            Box::pin(async move {
                println!("[OK] Logged in as {}", ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                println!("[OK] Slash commands registered globally");
                Ok(Data {
                    owners: owners_inner,
                    db: inner_db,
                    core_http_url: core_http_inner,
                })
            })
        })
        .build();

    let mut client = serenity::Client::builder(&config.token, intents)
        .framework(framework)
        .await
        .map_err(|e| BotError::Client(format!("Failed to create client: {}", e)))?;

    let shard_manager = client.shard_manager.clone();
    let http = client.http.clone();

    // Bot status rotation (XAUUSD price display)
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        let mut idx = 0;
        loop {
            interval.tick().await;
            let mut activities = vec![];
            if let Some(xau) = bot::services::market_ws::get_xauusd_display() {
                activities.push(ActivityData::custom(xau));
            }
            if !activities.is_empty() {
                let runners = shard_manager.runners.lock().await;
                for (_, runner) in runners.iter() {
                    runner.runner_tx.set_presence(
                        Some(activities[idx % activities.len()].clone()),
                        OnlineStatus::Online,
                    );
                }
                idx = (idx + 1) % activities.len();
            }
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Single unified WebSocket connection to core
    let bot_id = config.client_id.clone();
    start_core_ws_service(db_for_ws, http.clone(), config.core_ws_url.clone(), bot_id);
    println!(
        "[OK] Core WebSocket service started (connecting to {})",
        config.core_ws_url
    );

    client
        .start()
        .await
        .map_err(|e| BotError::Client(format!("Failed to initialize client: {}", e)))?;

    Ok(())
}
