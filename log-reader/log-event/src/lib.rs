use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};
use std::ops::Add;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEvent {
    pub ts: DateTime<Local>,
    pub level: String,
    pub service: String,
    pub latency: usize,
    pub endpoint: String,
}


#[derive(Debug)]
pub struct Stats {
    pub entries: usize,
    pub total_errors: usize,
    pub total_fatals: usize,
    pub average_latency: f64,
}

impl Stats {
    pub fn new() -> Stats {
        Stats {
            entries: 0,
            total_errors: 0,
            total_fatals: 0,
            average_latency: 0.,
        }
    }
}

impl Add for Stats {
    type Output = Self;
    fn add(self, other: &Self) -> Self {
        Stats {
            entries: self.entries + other.entries,
            total_errors: self.total_errors + other.total_errors,
            total_fatals: self.total_fatals + other.total_fatals,
            average_latency: self.average_latency * self.entries as f64 +
                other.average_latency * other.entries as f64,
        }
    }
}

impl Stats {
    pub fn to_vec(&self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        v.push(self.entries.to_string());
        v.push(format!("{:.2}%", (self.total_errors as f64 / self.entries as f64) * 100.));
        v.push(format!("{:.2}%", (self.total_fatals as f64 / self.entries as f64) * 100.));
        v.push(format!("{:.2}", self.average_latency));
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_stats() {
        let stats = Stats {
            entries: 500000,
            total_errors: 100000,
            total_fatals: 100000,
            total_latency: 5000000,
        };
        assert_eq!(format!("{}", stats),
            "
- Entries: 500000
- Error rate: 20.00%
- Fatal rate: 20.00%
- Average latency: 10.00
".to_string());
    }
}
