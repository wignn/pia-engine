use chrono::Utc;
use redis::RedisResult;
use serde_json::Value;
use tracing::warn;

#[derive(Clone)]
pub struct DeadLetterQueue {
    redis_client: redis::Client,
    stream_key: String,
    max_len: usize,
}

impl DeadLetterQueue {
    pub fn new(redis_client: redis::Client, stream_key: impl Into<String>, max_len: usize) -> Self {
        Self {
            redis_client,
            stream_key: stream_key.into(),
            max_len,
        }
    }

    pub async fn push(&self, pipeline: &str, item_id: &str, error: &str, payload: &Value) -> bool {
        let payload = match serde_json::to_string(payload) {
            Ok(payload) => payload,
            Err(err) => {
                warn!(error = %err, pipeline, item_id, "failed to serialize DLQ payload");
                return false;
            }
        };

        let mut conn = match self.redis_client.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(err) => {
                warn!(error = %err, pipeline, item_id, "failed to connect to redis for DLQ push");
                return false;
            }
        };

        let result: RedisResult<String> = redis::cmd("XADD")
            .arg(&self.stream_key)
            .arg("MAXLEN")
            .arg("~")
            .arg(self.max_len)
            .arg("*")
            .arg("pipeline")
            .arg(pipeline)
            .arg("item_id")
            .arg(item_id)
            .arg("error")
            .arg(error)
            .arg("payload")
            .arg(payload)
            .arg("failed_at")
            .arg(Utc::now().to_rfc3339())
            .arg("retry_count")
            .arg(0)
            .query_async(&mut conn)
            .await;

        match result {
            Ok(_) => true,
            Err(err) => {
                warn!(error = %err, pipeline, item_id, stream = %self.stream_key, "failed to push item to DLQ");
                false
            }
        }
    }
}
