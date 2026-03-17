use std::collections::HashMap;
use std::process;
use std::path::PathBuf;
use std::fs::File;
use std::error::Error;
use std::io::{BufReader, BufRead};

use log_event::{LogEvent, Stats};

use comfy_table::{Table, ContentArrangement, presets};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// name of the log file
    file: String,
}
fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("Error running the program: {e}");
        process::exit(1);
    };
}

fn track_line(every: i32, l: i32) {
    if l % every == 0 {
        println!("Passed line {l}");
    }
}

fn online_mean(average: f64, item: f64, item_count: f64) -> f64 {
    average + (item - average) / (item_count + 1.)
}

fn add_log_to_stats(log: &LogEvent, stats: &mut Stats) {
    stats.average_latency = online_mean(stats.average_latency,
        log.latency as f64,
        stats.entries as f64,
    );
    stats.entries += 1;
    if log.level == "error" {
        stats.total_errors += 1;
    }
    if log.level == "fatal" {
        stats.total_fatals += 1;
    }
}

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(&args.file);
    let mut reader = BufReader::new(File::open(path)?);
    let mut buffer = String::new();
    let mut line = 0;
    let mut data = HashMap::new();
    while let Ok(bytes) = reader.read_line(&mut buffer) {

        track_line(100000, line);
        if bytes == 0 { break; }

        let le: LogEvent = serde_json::from_str(&buffer)?;
        let stats: &mut Stats = data.entry(le.service.clone()).or_insert(Stats::new());

        add_log_to_stats(&le, stats);

        line += 1;
        buffer.clear();
    }
    let mut table = make_table(&data);
    table.set_footer(make_table(&data));
    println!("{table}");
    Ok(())
}

fn make_summary(data: &HashMap<String, Stats>) -> Vec<String> {
    let mut summary = Stats::new();
    for key in data.keys() {
        let stats = data.get(key).unwrap();
        summary = summary + stats;
    }
    let mut row: Vec<String> = vec!["Summary".to_string()];
    row.append(&mut summary.to_vec());
    row
}

fn make_table(data: &HashMap<String, Stats>) -> Table {
    let mut keys: Vec<&String> = data.keys().collect();
    keys.sort();

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    // tables are created with disabled content arrangement.
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["Service", "Entries", "Error rate", "Fatal rate",
        "Average latency"]);

    for key in keys {
        let mut row = vec![key.clone()];
        let stats = data.get(key).unwrap();
        let mut sv = stats.to_vec();
        row.append(&mut sv);
        table.add_row(row);
    }
    table
}
