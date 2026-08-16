use std::time::Duration;

use async_nats::{
    Client,
    jetstream::{
        self,
        consumer::{self, PullConsumer},
        stream::{self, RetentionPolicy},
    },
};
use bytes::Bytes;
use futures::StreamExt;

use crate::{app::Config, error::Result};

#[derive(Clone)]
pub struct NatsStore {
    client: Client,
    jetstream: jetstream::Context,
    consumer: PullConsumer,
    output_subject: String,
}

impl NatsStore {
    pub async fn connect(config: &Config) -> Result<Self> {
        let client = async_nats::connect(&config.nats_url).await?;
        let jetstream = jetstream::new(client.clone());

        jetstream
            .get_or_create_stream(stream::Config {
                name: config.jobs_stream.clone(),
                subjects: vec![config.input_subject.clone()],
                retention: RetentionPolicy::WorkQueue,
                ..Default::default()
            })
            .await?;
        jetstream
            .get_or_create_stream(stream::Config {
                name: config.results_stream.clone(),
                subjects: vec![config.output_subject.clone()],
                ..Default::default()
            })
            .await?;

        let jobs = jetstream.get_stream(&config.jobs_stream).await?;
        let consumer = jobs
            .get_or_create_consumer(
                &config.consumer_name,
                consumer::pull::Config {
                    durable_name: Some(config.consumer_name.clone()),
                    filter_subject: config.input_subject.clone(),
                    ack_wait: Duration::from_secs(120),
                    max_deliver: config.max_deliver,
                    max_ack_pending: config.workers as i64,
                    ..Default::default()
                },
            )
            .await?;

        Ok(Self {
            client,
            jetstream,
            consumer,
            output_subject: config.output_subject.clone(),
        })
    }

    pub async fn next_message(&self) -> Result<Option<jetstream::Message>> {
        let mut messages = self
            .consumer
            .fetch()
            .max_messages(1)
            .expires(Duration::from_secs(2))
            .messages()
            .await?;
        Ok(messages.next().await.transpose().unwrap())
    }

    pub async fn publish_result(&self, payload: Bytes) -> Result<()> {
        let ack = self
            .jetstream
            .publish(self.output_subject.clone(), payload)
            .await?;
        ack.await?;
        Ok(())
    }

    pub async fn flush(&self) -> Result<()> {
        self.client.flush().await?;
        Ok(())
    }
}
