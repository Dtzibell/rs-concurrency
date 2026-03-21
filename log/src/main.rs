use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::num::NonZero;
use std::thread;

use chrono::Local;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
#[serde(untagged)]
enum MapValues {
    String(String),
    UInt(u64),
}

const ENTRIES: usize = 4_500_000;
// there is a cap to how much faster the script runs based on how many threads
// are spawned. The softcap is about 15, where it runs in 6 seconds. Even with
// 200 threads it doesnt run much faster.
// fn format_ts(ldt: &DateTime<Local>) -> String {
//     format!(
//         "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
//         ldt.year(), ldt.month() as i32, ldt.day(),
//         ldt.hour(), ldt.minute(), ldt.second()
//     )
// }

fn main() -> Result<(), Box<dyn Error>> {
    let threads: usize = thread::available_parallelism()
        .unwrap_or(NonZero::new(8).unwrap())
        .get();
    println!("Continuing with {threads} threads");
    println!("Creating {ENTRIES} entries");
    let begin = std::time::Instant::now();
    let mut handles = vec![];

    // creates the directory, but does not care if it exitsts.
    let _ = std::fs::create_dir("logs/");
    for i in 0..threads {
        let handle = thread::spawn(move || {
            let mut rng = rand::rng();
            let services = [
                String::from("auth"),
                String::from("api"),
                String::from("gateway"),
                String::from("db-proxy"),
                String::from("billing"),
                String::from("inventory"),
                String::from("notification"),
                String::from("worker"),
            ];
            let level = [
                String::from("info"),
                String::from("error"),
                String::from("debug"),
                String::from("warn"),
                String::from("fatal"),
                String::from("trace"),
            ];
            let endpoint = [
                String::from("/login"),
                String::from("/users"),
                String::from("/health"),
                String::from("/signup"),
                String::from("/checkout"),
                String::from("/metrics"),
            ];
            let (min_ms, max_ms) = (10, 1000);
            let service_weight = WeightedIndex::new(
                (0..services.len())
                    .map(|_| rand::random_range(1..10))
                    .collect::<Vec<i32>>(),
            )
            .unwrap();
            let level_weight = WeightedIndex::new(
                (0..level.len())
                    .map(|_| rand::random_range(1..10))
                    .collect::<Vec<i32>>(),
            )
            .unwrap();
            let endpoint_weight = WeightedIndex::new(
                (0..level.len())
                    .map(|_| rand::random_range(1..10))
                    .collect::<Vec<i32>>(),
            )
            .unwrap();

            // creates a separate file for each of the threads
            let file = File::create(format!("logs/log-{}.json", i)).unwrap();
            let mut buf = BufWriter::new(file);

            for _ in 0..ENTRIES / threads {
                let mut map: HashMap<String, MapValues> = HashMap::new();
                map.insert(
                    "ts".to_string(),
                    MapValues::String(Local::now().to_string()),
                );
                map.insert(
                    "service".to_string(),
                    MapValues::String(services[service_weight.sample(&mut rng)].clone()),
                );
                map.insert(
                    "level".to_string(),
                    MapValues::String(level[level_weight.sample(&mut rng)].clone()),
                );
                map.insert(
                    "latency".to_string(),
                    MapValues::UInt(rand::random_range(min_ms..=max_ms)),
                );
                map.insert(
                    "endpoint".to_string(),
                    MapValues::String(endpoint[endpoint_weight.sample(&mut rng)].clone()),
                );
                serde_json::to_writer(&mut buf, &map).unwrap();
                buf.write_all(b"\n").unwrap();
            }
        });
        handles.push(handle);
    }
    for h in handles {
        h.join().unwrap();
    }

    // concatenates the files into one.
    let mut out = File::create("log.json").unwrap();
    for i in 0..threads {
        let path = format!("logs/log-{}.json", i);
        let mut file = File::open(&path).unwrap();
        let _ = std::io::copy(&mut file, &mut out).unwrap();
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_dir("logs/");
    }
    println!("Done in {}ms", begin.elapsed().as_millis());
    Ok(())
}
