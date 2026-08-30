use async_nats::jetstream::Context;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DeadLetter<'a> {
    pub source_subject: &'a str,
    pub error: &'a str,
    pub payload: &'a [u8],
}

pub async fn publish_dlq(
    context: &Context,
    dlq_subject: &str,
    source_subject: &str,
    error: &str,
    payload: &[u8],
) -> anyhow::Result<()> {
    let item = DeadLetter {
        source_subject,
        error,
        payload,
    };
    let body = serde_json::to_vec(&item)?;
    context
        .publish(dlq_subject.to_string(), body.into())
        .await?
        .await?;
    Ok(())
}
