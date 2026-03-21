use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
    process, time,
};

use log_event::{LogEvent, LogStats};

use clap::Parser;
use comfy_table::{Cell, Color, ContentArrangement, Table, presets};
use rayon::{prelude::*, str::ParallelString};

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
    let data = read_data(&path)?;
    let table = make_table(&data);
    println!("{table}");
    println!("Done in {}ms", start.elapsed().as_millis());
    Ok(())
}

fn read_data(path: &Path) -> Result<HashMap<String, LogStats>, Box<dyn Error>> {
    let buffer = std::fs::read_to_string(path)?;
    let data = buffer
        .par_lines()
        // fold is equivalent to foldl, thats why identity is HashMap
        .fold(HashMap::new, |mut h: HashMap<String, LogStats>, b: &str| {
            let le: LogEvent = serde_json::from_str(b)
                .unwrap_or_else(|err| panic!("Problem converting {b} to LogEvent: {err}"));
            h.entry(le.service.clone())
                .or_insert_with(LogStats::new)
                .document(&le);
            h
        })
        // Parallel folding produces a ParIter over hashmaps produced by separate
        // threads. This means that they need to be put back together. Reduce
        // is basically fold, but it takes two arguments that are of the same
        // type for OP and only outputs one item, in this case HashMap<String,
        // LogStats>.
        .reduce(
            HashMap::new,
            |mut h: HashMap<String, LogStats>, other: HashMap<String, LogStats>| {
                for (service, ls) in other.iter() {
                    *h.entry(service.to_string()).or_insert_with(LogStats::new) += ls;
                }
                h
            },
        );
    Ok(data)
}

fn make_table(data: &HashMap<String, LogStats>) -> Table {
    let mut keys: Vec<&String> = data.keys().collect();
    keys.sort();

    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "Service",
            "Entries",
            "Error rate",
            "Fatal rate",
            "Average latency",
        ]);

    let mut total_logs = LogStats::new();
    for key in keys {
        let stats = data.get(key).unwrap();

        let mut row = vec![key.clone()];
        let mut summary_vector = stats.summarize().vectorize();
        row.append(&mut summary_vector);
        table.add_row(row);

        total_logs += stats;
    }
    let mut summary_row = vec![Cell::new("Summary").fg(Color::Blue)];
    summary_row.append(
        &mut total_logs
            .summarize()
            .vectorize()
            // didnt realize that you need an Iterator to map :thinking:
            .iter()
            .map(|e| Cell::new(e).fg(Color::Blue))
            .collect::<Vec<Cell>>(),
    );
    table.add_row(summary_row);

    table
}
