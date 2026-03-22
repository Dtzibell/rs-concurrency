use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicUsize, Ordering},
    time,
};

use data_proc::{make_table, read_data};
use log_event::{LogEvent, LogStats};

use clap::Parser;
use comfy_table::{Cell, Color, ContentArrangement, Table, presets};
use rayon::{prelude::*, str::ParallelString};
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// name of the log file
    file: String,
}

fn main() {
    tracing_subscriber::fmt::init();
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
