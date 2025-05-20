use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use common::event_forwarding;
use rdkafka::{
    error::KafkaError,
    producer::{BaseProducer, BaseRecord, Producer},
    types::RDKafkaErrorCode,
    ClientConfig,
};
use crate::Result;

pub struct Publisher {
    topic: String,
    producer: BaseProducer,
    since_last_poll: AtomicUsize,
}

const POLL_INTERVAL: usize = 100;

impl Publisher {
    pub fn new(brokers: Vec<String>, topic: String) -> Result<Self> {
        let producer: BaseProducer = ClientConfig::new()
            .set("bootstrap.servers", &brokers.join(","))
            .set("api.version.request", "true")
            .set("message.timeout.ms", "5000")
            .create()?;

        Ok(Self {
            topic,
            producer,
            since_last_poll: AtomicUsize::new(0),
        })
    }

    pub fn send(&self, ev: &event_forwarding::Event, guild_id: u64) -> Result<()> {
        let marshalled = serde_json::to_vec(ev)?;
        let key = guild_id.to_string();

        let record = BaseRecord::to(&self.topic)
            .payload(&marshalled)
            .key(&key);

        if self.since_last_poll.fetch_add(1, Ordering::Relaxed) >= POLL_INTERVAL {
            self.producer.poll(Duration::from_millis(0));
            self.since_last_poll.store(0, Ordering::Relaxed);
        }

        match self.producer.send(record) {
            Ok(_) => Ok(()),
            Err((KafkaError::MessageProduction(RDKafkaErrorCode::QueueFull), _)) => {
                self.producer.poll(Duration::from_millis(10));
                Err(KafkaError::MessageProduction(RDKafkaErrorCode::QueueFull).into())
            }
            Err((e, _)) => Err(e.into()),
        }
    }

    pub fn flush(&self, timeout: Duration) {
        self.producer.flush(timeout);
    }
}