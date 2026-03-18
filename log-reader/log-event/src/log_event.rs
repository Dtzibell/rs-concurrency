use chrono::{DateTime, FixedOffset, TimeZone};
use serde::{Serialize, Deserialize};
use std::ops::Add;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LogEvent {
    pub ts: DateTime<FixedOffset>,
    pub level: String,
    pub service: String,
    pub latency: usize,
    // endpoint is not used under the current impl? 
    // e.g. /signup
    pub endpoint: String,
}

#[derive(Debug, PartialEq)]
pub struct LogStats {
    pub entries: usize,
    pub total_errors: usize,
    pub total_fatals: usize,
    pub total_latency: usize,
}

impl LogStats {
    pub fn new() -> LogStats {
        LogStats {
            entries: 0,
            total_errors: 0,
            total_fatals: 0,
            total_latency: 0,
        }
    }

    pub fn document(&mut self, le: &LogEvent) -> &Self {
        self.entries += 1;
        if le.level == String::from("error") {
            self.total_errors += 1;
        } else if le.level == String::from("fatal") {
            self.total_fatals += 1;
        }
        self.total_latency = self.total_latency + le.latency;
        self
    }

    pub fn summarize(&self) -> Summary {
        Summary {
            entries: self.entries,
            error_rate: self.total_errors as f64 / self.entries as f64,
            fatal_rate: self.total_fatals as f64 / self.entries as f64,
            average_latency: self.total_latency as f64 / self.entries as f64,
        }
    }
}
impl Add<&LogStats> for LogStats {
    type Output = Self;
    fn add(self, other: &Self) -> Self {
        LogStats {
            entries: self.entries + other.entries,
            total_errors: self.total_errors + other.total_errors,
            total_fatals: self.total_fatals + other.total_fatals,
            total_latency: self.total_latency + other.total_latency,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Summary {
    entries: usize,
    error_rate: f64,
    fatal_rate: f64,
    average_latency: f64,
}

impl Summary {
    pub fn new() -> Summary {
        Summary {
            entries: 0,
            error_rate: 0.,
            fatal_rate: 0.,
            average_latency: 0.,
        }
    }
    pub fn vectorize(&self) -> Vec<String> {
        vec![self.entries.to_string(),
            format!("{:.2}%", self.error_rate * 100.),
            format!("{:.2}%", self.fatal_rate * 100.),
            format!("{:.2}", self.average_latency)]
    }
}


impl From<LogStats> for Summary {
    fn from(value: LogStats) -> Self {
        Summary {
            entries: value.entries,
            error_rate: value.total_errors as f64 / value.entries as f64,
            fatal_rate: value.total_fatals as f64 / value.entries as f64,
            average_latency: value.total_latency as f64 / value.entries as f64,
        }
    }
}

fn online_mean(average: f64, new_item: f64, item_count: f64) -> f64{
    average + (new_item - average) / item_count
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_log_event() -> LogEvent {
        let local_date = FixedOffset::east_opt(3600).unwrap()
            .with_ymd_and_hms(2026, 2, 15, 9, 30, 0)
            .unwrap();
        LogEvent {
            ts: local_date,
            level: String::from("error"),
            service: String::from("api"),
            latency: 548,
            endpoint: String::from("/signup"),
        }
    }

    fn default_log_stats() -> LogStats {
        LogStats {
            entries: 2,
            total_errors: 1,
            total_fatals: 0,
            average_latency: 315.5,
        }
    }


    #[test]
    fn builds_default_log_stats() {
        assert_eq!(LogStats::new(),
            LogStats {
                entries: 0,
                total_errors: 0,
                total_fatals: 0,
                average_latency: 0.,
            });
    }

    #[test]
    fn calculates_latency_with_new_log_event() {
        let log = default_log_event();
        let mut stats = default_log_stats();
        stats.entries += 1; // because latency with does not increment entries
        let latency = stats.latency_with(&log);
        assert_eq!(latency, 393.);
    }

    #[test]
    fn documents_log_event() {
        let log = default_log_event();
        let mut stats = default_log_stats();
        assert_eq!(*stats.document(&log),
            LogStats {
                entries: 3,
                total_errors: 2,
                total_fatals: 0,
                average_latency: 393.,
            });
        let local_date = FixedOffset::east_opt(3600).unwrap()
            .with_ymd_and_hms(2026, 2, 15, 9, 30, 0)
            .unwrap();
        let another_log = LogEvent {
            ts: local_date,
            level: String::from("fatal"),
            service: String::from("api"),
            latency: 501,
            endpoint: String::from("/signup"),
        };
        assert_eq!(*stats.document(&another_log),
            LogStats {
                entries: 4,
                total_errors: 2,
                total_fatals: 1,
                average_latency: 420.,
            });
    }

    #[test]
    fn creates_summary() {
        let log = default_log_stats();
        assert_eq!(log.summarize(),
            Summary {
            entries: 2,
            error_rate: 0.5,
            fatal_rate: 0.,
            average_latency: 315.5,
        });
    }
}
