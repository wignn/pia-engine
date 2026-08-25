use async_nats::jetstream::Context;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DeadLetter<'a> {
    pub source_subject: &'a str,
    pub error: &'a str,
    pub payload: &'a [u8],
}

pub async fn publish(context: &Context, item: DeadLetter<'_>) -> anyhow::Result<()> {
    let body = serde_json::to_vec(&item)?;
    context
        .publish(
            atlsd_eventbus::subjects::MACRO_DLQ_V1.to_string(),
            body.into(),
        )
        .await?
        .await?;
    Ok(())
}
