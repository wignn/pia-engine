use tracing::warn;

/// Publish a config change event to Redis so the core service reloads tenant configs.
pub async fn publish_config_changed(redis: &Option<redis::Client>, prefix: &str) {
    let Some(client) = redis else { return };

    let channel = format!("{}:tenant:config_changed", prefix);
    let payload = chrono::Utc::now().to_rfc3339();

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
