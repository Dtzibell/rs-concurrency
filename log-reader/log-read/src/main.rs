use std::collections::HashMap;
use std::process;
use std::path::{PathBuf, Path};
use std::fs::File;
use std::error::Error;
use std::io::{BufReader, BufRead};
use std::time;
use std::thread;
use std::sync::{Arc, Mutex};

use log_event::{LogEvent, LogStats, Summary};

use comfy_table::{Table, ContentArrangement, presets, Color,
    Cell};
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
    let mut READER = Arc::new(Mutex::new(BufReader::new(File::open(path)?)));
    let mut DATA = Arc::new(Mutex::new(HashMap::new()));
    let mut handles = vec![];
    for i in 0..4 {
        let reader = Arc::clone(&mut READER);
        let data = Arc::clone(&mut DATA);
        let handle = thread::spawn(move || {
            let mut count = 0;
            let mut buffer = String::new();
            loop {
                {
                    let bytes = reader.lock().unwrap().read_line(&mut buffer);
                    if bytes.unwrap() == 0 { break; }
                }
                let le: LogEvent = serde_json::from_str(&buffer)
                .unwrap_or_else(|_| {
                        panic!("{buffer} could not be converted LogEvent")
                    });

                {
                    let mut map = data.lock().unwrap();
                    let mut stats = map.entry(le.service.clone())
                        .or_insert_with(LogStats::new);
                    stats.document(&le);
                }

                buffer.clear();

                count += 1;
                if count % print_every == 0 {
                    println!("Done with line {count}");
                }
            }
        });
        handles.push(handle);
    }
    for h in handles {
        h.join().unwrap();
    }
    Ok(Arc::try_unwrap(DATA).unwrap().into_inner().unwrap())
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
    let mut summary_row = vec![Cell::new("Summary").fg(Color::Blue)];
    summary_row.append(&mut total_logs
        .summarize()
        .vectorize()
        // didnt realize that you need an Iterator to map :thinking:
        .iter()
        .map(|e| Cell::new(e).fg(Color::Blue))
        .collect::<Vec<Cell>>()
    );
    table.add_row(summary_row);

    table
}
