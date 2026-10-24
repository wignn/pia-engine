use async_nats::jetstream::consumer::{self, AckPolicy, DeliverPolicy};
use async_nats::jetstream::stream;
use futures_util::StreamExt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

async fn connect_nats() -> Option<async_nats::Client> {
    let url = std::env::var("NATS_URL").ok()?;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    loop {
        match async_nats::connect(&url).await {
            Ok(client) => return Some(client),
            Err(_) if tokio::time::Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            Err(_) => panic!("NATS did not become available for the integration test"),
        }
    }
}

#[tokio::test]
async fn durable_consumer_recovers_raw_events_published_while_offline() {
    let Some(client) = connect_nats().await else {
        return;
    };
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let stream_name = format!("RAW_RECOVERY_{suffix}");
    let subject = format!("test.raw.{suffix}");
    let durable = format!("raw-recovery-{suffix}");
    let js = async_nats::jetstream::new(client.clone());
    let stream = js
        .get_or_create_stream(stream::Config {
            name: stream_name.clone(),
            subjects: vec![subject.clone()],
            ..Default::default()
        })
        .await
        .unwrap();

    js.publish(subject.clone(), "offline tick".into())
        .await
        .unwrap()
        .await
        .unwrap();

    let consumer = stream
        .get_or_create_consumer(
            &durable,
            consumer::pull::Config {
                durable_name: Some(durable.clone()),
                filter_subject: subject,
                deliver_policy: DeliverPolicy::All,
                ack_policy: AckPolicy::Explicit,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let mut messages = consumer.messages().await.unwrap();
    let message = tokio::time::timeout(Duration::from_secs(5), messages.next())
        .await
        .expect("offline event should be replayed")
        .expect("consumer stream should remain open")
        .unwrap();

    assert_eq!(message.message.payload.as_ref(), b"offline tick");
    message.ack().await.unwrap();
}

#[tokio::test]
async fn core_subscription_fans_out_to_every_gateway_instance() {
    let Some(client) = connect_nats().await else {
        return;
    };
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let subject = format!("test.fanout.{suffix}");
    let mut first_instance = client.subscribe(subject.clone()).await.unwrap();
    let mut second_instance = client.subscribe(subject.clone()).await.unwrap();

    client.publish(subject, "tick".into()).await.unwrap();

    for subscriber in [&mut first_instance, &mut second_instance] {
        let message = tokio::time::timeout(Duration::from_secs(5), subscriber.next())
            .await
            .expect("each realtime instance should receive a copy")
            .expect("subscription should remain open");
        assert_eq!(message.payload.as_ref(), b"tick");
    }
}
