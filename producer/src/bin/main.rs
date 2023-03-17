// Times
use core::time::Duration;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
// IO
use log::info;
use polars::prelude::*;
// Serialization
use prost::{Message};

// File Reader
pub mod hackernews {
    include!(concat!(env!("OUT_DIR"), "/hackernews.rs"));
}

fn read_parquets() -> LazyFrame {
    let args = ScanArgsParquet::default();
    let df = LazyFrame::scan_parquet("./data/hacker_news_full_*.parquet", args).unwrap();

    df.select([
        col("id"),
        col("time"),
        col("type"),
        col("title"),
        col("score"),
    ])
}

// Serialization
pub fn serialize_hackernews_row(row: hackernews::Row) -> Vec<u8> {
    let mut buf = Vec::new();
    row.encode(&mut buf).unwrap();
    buf
}

fn frame_into_protobuf(df: DataFrame) -> Vec<Vec<u8>> {
    let id_col = df.column("id").unwrap();
    let id_values: Vec<u32> = id_col
        .i64()
        .unwrap()
        .into_iter()
        .map(|x| x.unwrap() as u32)
        .collect();

    let time_col = df.column("time").unwrap();
    let time_values: Vec<Option<u64>> = time_col
        .i64()
        .unwrap()
        .into_iter()
        .map(|x| x.map(|x| x as u64))
        .collect();

    let type_col = df.column("type").unwrap();
    let type_values: Vec<hackernews::RowType> = type_col
        .utf8()
        .unwrap()
        .into_iter()
        .map(|x| {
            let x = x.unwrap();
            if x == "story" {
                hackernews::RowType::Story
            } else {
                hackernews::RowType::Comment
            }
        })
        .collect();

    let title_col = df.column("title").unwrap();
    let title_values: Vec<Option<String>> = title_col
        .utf8()
        .unwrap()
        .into_iter()
        .map(|x| x.map(|x| x.to_string()))
        .collect();

    let score_col = df.column("score").unwrap();
    let score_values: Vec<Option<u32>> = score_col
        .i64()
        .unwrap()
        .into_iter()
        .map(|x| x.map(|x| x as u32))
        .collect();

    let mut protobuf_messages = Vec::new();
    for idx in 0..id_values.len() {
        let id: u32 = id_values[idx];
        let timestamp: u64 = match &time_values[idx] {
            Some(x) => *x,
            None => 0,
        };
        let r#type = type_values[idx];
        let title = match &title_values[idx] {
            Some(x) => x,
            None => "",
        };
        let score: u32 = match score_values[idx] {
            Some(x) => x,
            None => 0,
        };
        let row = hackernews::Row {
            id: id,
            timestamp: timestamp,
            r#type: r#type as i32,
            title: title.to_string(),
            score: score,
        };
        let buf = serialize_hackernews_row(row);
        protobuf_messages.push(buf);
    }
    protobuf_messages
}

// Producer

pub struct RedpandaProducer {
    timeout_duration: Duration,
    producer: FutureProducer,
}

impl RedpandaProducer {
    pub fn new() -> Self {
        let timeout_duration = Duration::from_secs(1);
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", "localhost:9092")
            .set(
                "message.timeout.ms",
                &timeout_duration.as_millis().to_string(),
            )
            .create()
            .expect("Producer creation error");
        Self {
            timeout_duration: timeout_duration,
            producer: producer,
        }
    }

    pub async fn send(&mut self, topic: &str, value: &str) {
        let record = FutureRecord::to(&topic).payload(value).key("hackernews");
        let delivery_status = self.producer.send(record, self.timeout_duration).await;
        info!("Delivery status: {:#?}", delivery_status)
    }

    pub async fn send_protobuf(&mut self, topic: &str, value: &Vec<u8>) {
        let record = FutureRecord::to(&topic).payload(value).key("hackernews");
        let delivery_status = self.producer.send(record, self.timeout_duration).await;
        info!("Delivery status: {:#?}", delivery_status)
    }
}

#[tokio::main]
async fn main() {
    // Logging config with env_logger
    env_logger::init();

    // Read data
    let df: DataFrame = read_parquets().collect().unwrap();
    let col_names: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|x| x.to_string())
        .collect();
    info!("Column Names: {:#?}", col_names);

    // Create producer
    let mut producer = RedpandaProducer::new();
    let topic = "hello-world-topic";
    producer.send(topic, "Hello World!").await;
    // value = "Hello World! First row: {first_row}"
    // let value = format!("Hello World! First row: {:#?}", first_row);

    let topic: &str = "hackernews-topic";
    let protobuf_messages = frame_into_protobuf(df);
    for message in protobuf_messages {
        producer.send_protobuf(topic, &message).await;
        info!("Sent: {:#?}", message);
    }
}
