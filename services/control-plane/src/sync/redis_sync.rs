use atlsd_eventbus::{subjects, EventBusMode, EventPublisher, NatsPublisher};
use tracing::warn;

pub async fn publish_config_changed_for_user(
    redis: &Option<redis::Client>,
    prefix: &str,
    user_id: Option<uuid::Uuid>,
) {
    let payload = serde_json::json!({
        "changed_at": chrono::Utc::now().to_rfc3339(),
        "user_id": user_id,
    });
    publish_redis_config_changed(redis, prefix, &payload).await;
    publish_nats_config_changed(&payload).await;
}

async fn publish_redis_config_changed(
    redis: &Option<redis::Client>,
    prefix: &str,
    payload: &serde_json::Value,
) {
    let Some(client) = redis else { return };

    let channel = format!("{}:tenant:config_changed", prefix);
    let payload = payload.to_string();

    match client.get_multiplexed_async_connection().await {
        Ok(mut conn) => {
            let r: redis::RedisResult<i64> = redis::cmd("PUBLISH")
                .arg(&channel)
                .arg(&payload)
                .query_async(&mut conn)
                .await;
            if let Err(e) = r {
                warn!(error = %e, "failed to publish config_changed to redis");
            }
        }
        Err(e) => {
            warn!(error = %e, "redis connection failed for config sync");
        }
    }
}

async fn publish_nats_config_changed(payload: &serde_json::Value) {
    let mode = EventBusMode::from_env_value(
        &std::env::var("EVENTBUS_MODE").unwrap_or_else(|_| "redis".to_string()),
    );
    if !matches!(mode, EventBusMode::Nats | EventBusMode::Dual) {
        return;
    }

    let url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    match NatsPublisher::connect(&url).await {
        Ok(publisher) => {
            if let Err(err) = publisher
                .publish_json(subjects::TENANT_CONFIG_CHANGED_V1, payload)
                .await
            {
                warn!(error = %err, "failed to publish config_changed to NATS");
            }
        }
        Err(err) => warn!(error = %err, url = %url, "failed to connect to NATS for config sync"),
    }
}
