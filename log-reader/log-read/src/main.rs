use clap::Parser;
use std::process;
use std::path::PathBuf;
use std::fs::File;
use std::error::Error;
use std::io::{BufReader, BufRead};
use log_event::LogEvent;

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
    for i in 0..10 {
        let mut buffer = String::new();
        reader.read_line(&mut buffer)?;
        println!("{buffer}");
        let le: LogEvent = serde_json::from_str(&buffer)?;
        println!("{:#?}", le);
    }
    Ok(())
}
