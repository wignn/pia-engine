use crate::commands::Data;
use crate::config::Config;
use crate::error::BotError;
use crate::handlers::{handle_event, on_error};
use crate::repository::DbPool;
use poise::serenity_prelude::UserId;
use serenity::all::{GatewayIntents, Http, ShardManager};
use std::collections::HashSet;
use std::sync::Arc;

pub struct BotClient {
    pub client: serenity::Client,
    pub shard_manager: Arc<ShardManager>,
    pub http: Arc<Http>,
}

pub async fn build_bot_client(config: &Config, db: DbPool) -> Result<BotClient, BotError> {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_MEMBERS;

    let owner_id = config
        .client_id
        .parse::<u64>()
        .map_err(|e| BotError::Config(format!("CLIENT_ID must be a valid u64: {}", e)))?;

    let mut owners = HashSet::new();
    owners.insert(UserId::new(owner_id));

    let owners_for_setup = owners.clone();
    let db_for_setup = db.clone();
    let api_http_url = config.api_http_url.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: crate::commands::all(),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            on_error: |error| Box::pin(on_error(error)),
            event_handler: |ctx, event, _framework, data| Box::pin(handle_event(ctx, event, data)),
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            let db = db_for_setup.clone();
            let owners = owners_for_setup.clone();
            let api_http_url = api_http_url.clone();

            Box::pin(async move {
                println!("[OK] Logged in as {}", ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                println!("[OK] Slash commands registered globally");

                Ok(Data {
                    owners,
                    db,
                    api_http_url,
                })
            })
        })
        .build();

    let client = serenity::Client::builder(&config.token, intents)
        .framework(framework)
        .await
        .map_err(|e| BotError::Client(format!("Failed to create client: {}", e)))?;

    let shard_manager = client.shard_manager.clone();
    let http = client.http.clone();

    Ok(BotClient {
        client,
        shard_manager,
        http,
    })
}
