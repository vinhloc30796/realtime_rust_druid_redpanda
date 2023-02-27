// Times
use core::time::Duration;
use kafka::producer::{Producer, Record, RequiredAcks};
// IO
use polars::frame::row::Row;
use polars::prelude::*;
// Serialization
// use crate::prost;
use prost::{Enumeration, Message};
use std::iter::zip;

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

// File Reader
#[derive(Clone, Copy, PartialEq, Eq, Debug, Enumeration)]
pub enum HackerNewsType {
    Story = 0,
    Comment = 1,
}

#[derive(Clone, PartialEq, Message)]
struct HackerNewsRow {
    #[prost(uint32, tag = "1")]
    pub id: u32,
    #[prost(uint32, tag = "2")]
    pub timestamp: u32,
    #[prost(enumeration = "HackerNewsType", tag = "3")]
    pub r#type: i32,
    #[prost(string, tag = "4")]
    pub title: String,
    #[prost(uint32, tag = "5")]
    pub score: u32,
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

fn frame_into_protobuf(df: DataFrame) -> Vec<Vec<u8>> {
    let id_col = df.column("id").unwrap();
    let id_values: Vec<u32> = id_col
        .i64()
        .unwrap()
        .into_iter()
        .map(|x| x.unwrap() as u32)
        .collect();

    let time_col = df.column("time").unwrap();
    let time_values: Vec<Option<u32>> = time_col
        .i64()
        .unwrap()
        .into_iter()
        .map(|x| x.map(|x| x as u32))
        .collect();

    let type_col = df.column("type").unwrap();
    let type_values: Vec<HackerNewsType> = type_col
        .utf8()
        .unwrap()
        .into_iter()
        .map(|x| {
            let x = x.unwrap();
            if x == "story" {
                HackerNewsType::Story
            } else {
                HackerNewsType::Comment
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
        let id = id_values[idx];
        let timestamp = match &time_values[idx] {
            Some(x) => *x,
            None => 0,
        };
        let r#type = type_values[idx];
        let title = match &title_values[idx] {
            Some(x) => x,
            None => "",
        };
        let score = match &score_values[idx] {
            Some(x) => x,
            None => &0,
        };
        let message = HackerNewsRow {
            id: id,
            timestamp: timestamp,
            r#type: r#type as i32,
            title: title.to_string(),
            score: *score,
        };
        let mut buf = Vec::new();
        message.encode(&mut buf).unwrap();
        protobuf_messages.push(buf);
    }
    protobuf_messages
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
    let df: DataFrame = read_parquets().collect().unwrap();
    let col_names: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|x| x.to_string())
        .collect();
    println!("Column Names: {:#?}", col_names);

    // Create producer
    // let mut producer = RedpandaProducer::new();
    let topic = "hello-world-topic";
    // value = "Hello World! First row: {first_row}"
    // let value = format!("Hello World! First row: {:#?}", first_row);

    let protobuf_messages = frame_into_protobuf(df);
}
