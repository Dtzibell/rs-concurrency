use clap::Parser;
use std::collections::HashMap;
use std::process;
use std::path::PathBuf;
use std::fs::File;
use std::error::Error;
use std::io::{BufReader, BufRead};
use log_event::{LogEvent, Stats};

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// name of the log file
    file: String,
}
fn main() -> Result<(), Box<dyn Error>>{
    let args = Args::parse();
    let path = PathBuf::from(&args.file);
    if !path.exists() {
        eprintln!("{} was not found", args.file);
        process::exit(1);
    }
    let mut reader = BufReader::new(File::open(path)?);
    let mut buffer = String::new();
    let mut i = 0;
    let mut data = HashMap::new();
    'top: while let Ok(bytes) = reader.read_line(&mut buffer) {
        if bytes == 0 { break 'top; }
        if i % 25000 == 0 {
            println!("{i}");
        }
        let le: LogEvent = serde_json::from_str(&buffer)?;
        let stats = data.entry(le.service).or_insert(Stats::new());
        stats.total_latency += le.latency;
        // println!("Service: {:#?}, Average = {}, Instance = {}", stats, stats.average_latency, le.latency);
        stats.entries += 1;
        if le.level == "error" {
            stats.errors += 1;
        }
        if le.level == "fatal" {
            stats.fatals += 1;
        }

        i += 1;
        buffer.clear();
    } 
    for (key, value) in &mut data {
        value.average_latency = value.total_latency as f64 / value.entries as f64;
    }
    println!("{data:#?}");
    Ok(())
}
