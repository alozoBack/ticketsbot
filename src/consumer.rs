use common::event_forwarding::Event;
use rdkafka::{
    consumer::{Consumer as KafkaConsumer, StreamConsumer},
    ClientConfig, Message,
};
use tracing::warn;
use crate::Result;

pub struct Consumer {
    consumer: StreamConsumer,
}

impl Consumer {
    pub fn new(brokers: Vec<String>, topic: String, group_id: String) -> Result<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &brokers.join(","))
            .set("group.id", &group_id)
            .set("enable.auto.commit", "true")
            .set("session.timeout.ms", "6000")
            .set("api.version.request", "true")
            .create()?;

        consumer.subscribe(&[&topic])?;

        Ok(Self { consumer })
    }

    pub async fn recv(&self) -> Result<Event> {
        let msg = self.consumer.recv().await?;

        let payload = match msg.payload_view::<[u8]>() {
            Some(Ok(bytes)) => bytes,
            Some(Err(e)) => {
                warn!("Error deserializing payload: {e}");
                return self.recv().await;
            }
            None => {
                warn!("Received message with no payload. Ignoring.");
                return self.recv().await;
            }
        };

        serde_json::from_slice(payload).map_err(Into::into)
    }
}
