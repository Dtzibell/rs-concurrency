use std::collections::HashMap;
use std::process;
use std::path::{PathBuf, Path};
use std::fs::File;
use std::error::Error;
use std::io::{BufReader, BufRead};
use std::time;

use log_event::{LogEvent, LogStats, Summary};

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
    let start = time::Instant::now();
    let path = PathBuf::from(&args.file);
    let data = read_data(&path, 100000)?;
    let table = make_table(&data);
    println!("{table}");
    println!("Done in {}ms", start.elapsed().as_millis());
    Ok(())
}

// set print_every to u64::MAX if dont want to print.
fn read_data(path: &Path, print_every: u64) -> Result<HashMap<String, LogStats>, Box<dyn Error>> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut buffer = String::new();
    let mut count = 0;
    let mut data = HashMap::new();
    loop {
        let bytes = reader.read_line(&mut buffer);
        if bytes.unwrap() == 0 { break; }
        let le: LogEvent = serde_json::from_str(&buffer)?;

        data.entry(le.service.clone())
            .or_insert_with(LogStats::new)
            .document(&le);

        buffer.clear();

        count += 1;
        if count % print_every == 0 {
            println!("Done with line {count}");
        }
    }
    Ok(data)
}


fn make_table(data: &HashMap<String, LogStats>) -> Table {
    let mut keys: Vec<&String> = data.keys().collect();
    keys.sort();

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Service", "Entries", "Error rate", "Fatal rate", 
            "Average latency"]);

    let mut total_logs = LogStats::new();
    for key in keys {
        let stats = data.get(key).unwrap();

        let mut row = vec![key.clone()];
        let mut summary_vector = stats.summarize().vectorize();
        row.append(&mut summary_vector);
        table.add_row(row);

        total_logs = total_logs + stats;
    }
    let mut summary_row = vec!["Summary".to_string()];
    summary_row.append(&mut total_logs.summarize().vectorize());
    table.add_row(summary_row);

    table
}
