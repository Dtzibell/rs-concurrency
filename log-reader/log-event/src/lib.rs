use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEvent {
    pub ts: DateTime<Local>,
    pub level: String,
    pub service: String,
    pub latency: u64,
    pub endpoint: String,
}

#[derive(Debug)]
pub struct Stats {
    pub entries: u64,
    pub errors: usize,
    pub fatals: usize,
    pub total_latency: u64,
    pub average_latency: f64,
}

impl Stats {
    pub fn new() -> Stats {
        Stats {
            entries: 0,
            errors: 0,
            fatals: 0,
            total_latency: 0,
            average_latency: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
