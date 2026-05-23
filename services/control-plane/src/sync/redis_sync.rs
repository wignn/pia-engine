use tracing::warn;

pub async fn publish_config_changed_for_user(
    redis: &Option<redis::Client>,
    prefix: &str,
    user_id: Option<uuid::Uuid>,
) {
    let Some(client) = redis else { return };

    let channel = format!("{}:tenant:config_changed", prefix);
    let payload = serde_json::json!({
        "changed_at": chrono::Utc::now().to_rfc3339(),
        "user_id": user_id,
    })
    .to_string();

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
