use comfy_table::{Cell, Color, ContentArrangement, Table, presets};
use serde_json;
use std::{
    mem::drop,
    collections::HashMap, 
    sync::atomic::{Ordering, AtomicUsize},
    error::Error,
    path::Path,
    thread,
    io::{BufRead, BufReader},
    fs::File,
};
use log_event::{LogStats, LogEvent};
use rayon::prelude::*;
use crossbeam::channel::{bounded, Sender, Receiver};
use tracing::{info, instrument, info_span};

fn read_and_send_vec(mut reader: impl BufRead, capacity: usize, transmitter: Sender<Vec<String>>) {
    let mut v = Vec::with_capacity(capacity);
    let mut buffer = String::new();
    let mut batches_sent = 0;
    while let Ok(m) = reader.read_line(&mut buffer) {
        if m == 0 {
            transmitter.send(v).unwrap();
            break;
        }
        if v.len() == capacity {
            transmitter.send(v).unwrap();
            v = Vec::with_capacity(capacity);
        }
        v.push(buffer);
        buffer = String::new();
        batches_sent += 1;
        // info!(batches_sent, "reader sent batch");
    }
}

fn parse_and_send_stats(receiver: Receiver<Vec<String>>, transmitter: Sender<HashMap<String, LogStats>>) {
    // let thread_id = thread::current().id();
    let mut batches = 0;
    let mut h = HashMap::new();
    // let start = std::time::Instant::now();
    while let Ok(m) = receiver.recv() {
        for s in m.iter() {
            // let _span = info_span!("parse_batch", ?thread_id, batch = batches).entered();
            let le: LogEvent = serde_json::from_str(&s).expect("Could not parse {s} to LogEvent");
            document_log_event(&mut h, le);
        }
        batches += 1;
        // print!("[{:?}] batches: {}, elapsed: {:.1}s", 
        //     thread_id, batches, start.elapsed().as_secs_f32());
        if batches % 20 == 0 {
            transmitter.send(h).unwrap();
            h = HashMap::new();
        }
        // info!(?thread_id, batches, "parser done");
    }
    if !h.is_empty() {
        transmitter.send(h).unwrap();
    }
}

fn aggregate(receiver: Receiver<HashMap<String, LogStats>>) -> HashMap<String, LogStats> {
    let mut data = HashMap::new();
    let mut count = 0;
    while let Ok(h) = receiver.recv() {
        for (service, ls) in h.iter() {
            *data.entry(service.clone()).or_insert_with(LogStats::new) += ls;
        }
        count += 1;
        // info!(count, "aggregated chunk");
    }
    // info!(count, "aggregation done");
    data
}

pub fn read_data(path: &Path, threads: usize) -> Result<HashMap<String, LogStats>, Box<dyn Error>> {
    let buffer = String::new();
    let reader = BufReader::with_capacity(128*1024,File::open(path)?);

    let (reader_t, parser_r) = bounded::<Vec<String>>(32);
    let (parser_t, aggregator_r) = bounded::<HashMap<String, LogStats>>(32);

    let reader = thread::spawn(move || {
        read_and_send_vec(reader, 1024, reader_t);
    });

    let mut parsers = vec![];
    for i in 0..threads {
        let parser_rx = parser_r.clone();
        let parser_tx = parser_t.clone();
        let parser = thread::spawn(move|| {
            parse_and_send_stats(parser_rx, parser_tx);
        });
        parsers.push(parser);
    }
    drop(parser_r);
    drop(parser_t);
    
    let aggregator = thread::spawn(move || {
        aggregate(aggregator_r)
    });
    
    reader.join().unwrap();
    for parser in parsers {
        parser.join().unwrap();
    }
    let data = aggregator.join().unwrap();
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
