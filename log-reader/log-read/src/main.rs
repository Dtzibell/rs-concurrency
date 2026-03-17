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


fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(&args.file);
    let data = read_data(path, 100000);
    let mut table = make_table(&data);
    println!("{table}");
    Ok(())
}

// set print_every to u64::MAX if dont want to print.
fn read_data(path: &Path, print_every: u64) -> HashMap<String, Stats> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut buffer = String::new();
    let mut count = 0;
    let mut data = HashMap::new();
    loop {
        let bytes = reader.read_line(&mut buffer);
        if bytes.unwrap() == 0 { break; }
        let le: LogEvent = serde_json::from_str(&buffer)?;

        data.entry(le.service)
            .or_insert_with(Stats::new)
            .document(&le);

        buffer.clear();

        line += 1;
        if count % print_every == 0 {
            println!("Done with line {count}");
        }
    }
    data
}


fn make_table(data: &HashMap<String, Stats>) -> Table {
    let mut keys: Vec<&String> = data.keys().collect();
    keys.sort();

    let mut table = Table::new()
        .load_preset(presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Service", "Entries", "Error rate", "Fatal rate", 
            "Average latency"]);

    for key in keys {
        let mut row = vec![key.clone()];
        let stats = data.get(key).unwrap();
        let mut sv = stats.summarize();
        row.append(&mut sv);
        table.add_row(row);
    }
    table
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
