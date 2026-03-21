use comfy_table::{Cell, Color, ContentArrangement, Table, presets};
use serde_json;
use std::collections::HashMap;
use std::sync::atomic::{Ordering, AtomicUsize};
use log_event::{LogStats, LogEvent};
use std::error::Error;
use std::path::Path;
use rayon::prelude::*;

pub fn read_data(path: &Path) -> Result<HashMap<String, LogStats>, Box<dyn Error>> {
    let count = AtomicUsize::new(0);
    let buffer = std::fs::read_to_string(path)?;
    let data = buffer
        .par_lines()
        // fold is equivalent to foldl, thats why identity is HashMap
        .fold(HashMap::new, |mut h: HashMap<String, LogStats>, b: &str| {
            let le: LogEvent = serde_json::from_str(b)
                .unwrap_or_else(|err| panic!("Problem converting {b} to LogEvent: {err}"));
            document_log_event(&mut h, le);
            increment_line_count(&count);
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
fn document_log_event(h: &mut HashMap<String, LogStats>, le: LogEvent) {
    h.entry(le.service.clone())
        .or_insert_with(LogStats::new)
        .document(&le);
}
fn increment_line_count(count: &AtomicUsize) {
    count.fetch_add(1, Ordering::Relaxed);
    let line_count = count.load(Ordering::Relaxed);
    if line_count % 500000 == 0 {
        println!("Finished with line {line_count}");
    }
}
pub fn make_table(data: &HashMap<String, LogStats>) -> Table {
    let mut keys: Vec<&String> = data.keys().collect();
    keys.sort();

    let mut table = new_table();
    table.set_header(vec![
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
    let summary_row = make_summary(&mut total_logs);
    table.add_row(summary_row);

    table
}

fn new_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}

fn make_summary(logs: &mut LogStats) -> Vec<Cell> {
    let mut summary_row = vec![Cell::new("Summary").fg(Color::Blue)];
    summary_row.append(
        &mut logs
            .summarize()
            .vectorize()
            // didnt realize that you need an Iterator to map :thinking:
            .iter()
            .map(|e| Cell::new(e).fg(Color::Blue))
            .collect::<Vec<Cell>>(),
    );
    summary_row
}
