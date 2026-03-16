use std::collections::HashMap;
use datetime::{LocalDateTime, Instant, TimePiece, DatePiece};
use rand::prelude::*;
use serde::Serialize;
use std::io::{BufWriter, Write};
use std::fs::File;
use std::error::Error;

#[derive(Serialize)]
#[serde(untagged)]
enum MapValues {
    String(String),
    UInt(u64),
}

const ENTRIES: usize = 5000000;
fn main() -> Result<(), Box<dyn Error>>{
    let services = [String::from("auth"), String::from("api"),
        String::from("gateway"), String::from("db-proxy"),
        String::from("billing"), String::from("inventory"),
        String::from("notification"), String::from("worker")];
    let level = [String::from("info"), String::from("error"),
        String::from("debug"), String::from("warn"), String::from("fatal"),
        String::from("trace")];
    let endpoint = [String::from("/login"), String::from("/users"),
        String::from("/health"), String::from("/signup"),
        String::from("/checkout"), String::from("/metrics")];
    let (min_ms, max_ms) = (10, 1000);
    // let ts = [LocalDateTime::from_instant(Instant::now()); ENTRIES];
    // println!("{}", format_ts(&ts[1]));
    let mut rng = rand::rng();
    // for i in 0..10 {
    //     println!("{:?}", services.choose(&mut rng).unwrap());
    // }
    let mut data: Vec<HashMap<String, MapValues>> = vec![];
    let mut buf = BufWriter::new(File::create("log.json")?);
    for i in 0..ENTRIES {
        let mut map: HashMap<String, MapValues> = HashMap::new();
        map.insert("ts".to_string(),
            MapValues::String(format_ts(&LocalDateTime::from_instant(Instant::now()))));
        map.insert("service".to_string(),
            MapValues::String(services.choose(&mut rng).unwrap().to_string()));
        map.insert("level".to_string(),
            MapValues::String(level.choose(&mut rng).unwrap().to_string()));
        map.insert("latency".to_string(),
            MapValues::UInt(rand::random_range(min_ms..=max_ms)));
        map.insert("endpoint".to_string(),
            MapValues::String(endpoint.choose(&mut rng).unwrap().to_string()));
        let j = serde_json::to_writer(&mut buf, &map)?;
        buf.write_all(b"\n")?;
    }
    Ok(())
}

fn format_ts(ldt: &LocalDateTime) -> String {
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        ldt.year(), ldt.month() as i32, ldt.day(),
        ldt.hour(), ldt.minute(), ldt.second()
    )
}
