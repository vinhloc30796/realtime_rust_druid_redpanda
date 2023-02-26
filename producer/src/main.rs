// Times
use core::time::Duration;
use kafka::producer::{Producer, Record, RequiredAcks};
// IO
use polars::prelude::*;

// File Reader

fn read_parquets() -> LazyFrame {
    let args = ScanArgsParquet::default();
    let df = LazyFrame::scan_parquet("./data/hacker_news_full_*.parquet", args).unwrap();

    df
}

// Producer

pub struct RedpandaProducer {
    producer: Producer,
}

impl RedpandaProducer {
    pub fn new() -> Self {
        let timeout_duration = Duration::from_secs(1);
        let producer = Producer::from_hosts(vec!["localhost:9092".to_owned()])
            .with_ack_timeout(timeout_duration)
            .with_required_acks(RequiredAcks::One)
            .create()
            .unwrap();
        Self { producer: producer }
    }

    pub fn send(&mut self, topic: &str, value: &str) {
        let record = Record::from_value(topic, value);
        self.producer.send(&record).unwrap();
    }
}

fn main() {
    // Read data
    let df = read_parquets().collect().unwrap();
    let first_row = df.get_row(0).unwrap();

    // Create producer
    let mut producer = RedpandaProducer::new();
    let topic = "hello-world-topic";
    // value = "Hello World! First row: {first_row}"
    let value = format!("Hello World! First row: {:#?}", first_row);

    // Loop 5 times
    for _ in 0..5 {
        let current_time_str = chrono::Local::now().to_string();
        let timed_value = format!("{}: {}", current_time_str, value);
        producer.send(topic, &timed_value);
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
