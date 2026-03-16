use std::collections::HashMap;
use chrono::{Local, DateTime};
use rand::prelude::*;
use serde::Serialize;
use std::io::{BufWriter, Write};
use std::fs::File;
use std::error::Error;
use std::thread;
use std::sync::{Arc, RwLock};

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
const THREADS: usize = 100;
// fn format_ts(ldt: &DateTime<Local>) -> String {
//     format!(
//         "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
//         ldt.year(), ldt.month() as i32, ldt.day(),
//         ldt.hour(), ldt.minute(), ldt.second()
//     )
// }

fn main() -> Result<(), Box<dyn Error>>{
    let begin = std::time::Instant::now();
    let mut handles = vec![];

    // creates the directory, but does not care if it exitsts.
    std::fs::create_dir("logs/");
    for i in 0..THREADS {
        let handle = thread::spawn(move || {
            let mut rng = rand::rng();
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

            // creates a separate file for each of the threads
            let file = File::create(&format!("logs/log-{}.json",i)).unwrap();
            let mut buf = BufWriter::new(file);

            for j in 0..ENTRIES/THREADS {
                let mut map: HashMap<String, MapValues> = HashMap::new();
                map.insert("ts".to_string(),
                    MapValues::String(Local::now().to_string()));
                map.insert("service".to_string(),
                    MapValues::String(services.choose(&mut rng).unwrap().to_string()));
                map.insert("level".to_string(),
                    MapValues::String(level.choose(&mut rng).unwrap().to_string()));
                map.insert("latency".to_string(),
                    MapValues::UInt(rand::random_range(min_ms..=max_ms)));
                map.insert("endpoint".to_string(),
                    MapValues::String(endpoint.choose(&mut rng).unwrap().to_string()));
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
    for i in 0..THREADS {
        let path = format!("logs/log-{}.json", i);
        let mut file = File::open(&path).unwrap();
        std::io::copy(&mut file, &mut out).unwrap();
        std::fs::remove_file(path);
        std::fs::remove_dir("logs/");
    }
    let end = std::time::Instant::now();
    println!("Done in {}ms", begin.elapsed().as_millis());
    Ok(())
}
