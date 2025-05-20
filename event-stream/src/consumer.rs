use common::event_forwarding::Event;
use rdkafka::{
    consumer::StreamConsumer,
    ClientConfig, Message,
};
use serde_json;
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
            .set("auto.offset.reset", "earliest")
            .set("api.version.request", "true")
            .create()?;

        consumer.subscribe(&[&topic])?;

        Ok(Self { consumer })
    }

    pub async fn recv(&self) -> Result<Event> {
        loop {
            let msg = self.consumer.recv().await?;

            match msg.payload_view::<[u8]>() {
                Some(Ok(payload)) => {
                    return serde_json::from_slice(payload).map_err(Into::into);
                }
                Some(Err(e)) => {
                    warn!("Error deserializing payload: {:?}", e);
                    continue;
                }
                None => {
                    warn!("Received message with no payload. Ignoring.");
                    continue;
                }
            }
        }
    }
}
