use chrono::{DateTime, Local, FixedOffset};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEvent {
    pub ts: DateTime<Local>,
    pub level: String,
    pub service: String,
    pub latency: usize,
    // endpoint is not used under the current impl? 
    // e.g. /signup
    pub endpoint: String,
}

#[derive(Debug)]
pub struct LogStats {
    pub entries: usize,
    pub total_errors: usize,
    pub total_fatals: usize,
    pub average_latency: f64,
}

impl LogStats {
    pub fn new() -> LogStats {
        LogStats {
            entries: 0,
            total_errors: 0,
            total_fatals: 0,
            average_latency: 0.,
        }
    }

    pub fn document(&mut self, le: &LogEvent) {
        self.entries += 1;
        if le.level == String::from("error") {
            self.total_errors += 1;
        } else if le.level == String::from("fatal") {
            self.total_fatals += 1;
        }
        self.average_latency = self.latency_with(le);
    }

    fn latency_with(&self, le: &LogEvent) -> f64 {
        online_mean(self.average_latency as f64,
            le.latency as f64,
            self.entries as f64)
    }

    pub fn summarize(&self) -> Summary {
        Summary {
            entries: self.entries,
            error_rate: self.total_errors as f64 / self.entries as f64,
            fatal_rate: self.total_fatals as f64 / self.entries as f64,
            average_latency: self.average_latency,
        }
    }
}

struct Summary {
    entries: i32,
    error_rate: f64,
    fatal_rate: f64,
    average_latency: f64,
}

impl Summary {
    fn vectorize() -> Vec<String> {
        unimplemented!();
    }
}

impl From<Stats> for Summary {
    fn from(value: Stats) -> Self {
        unimplemented!();
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
        let stats = default_log_stats();
        let latency = stats.latency_with(&log);
        assert_eq!(latency, 393);
    }

    #[test]
    fn documents_log_event() {
        let log = default_log_event();
        let stats = default_log_stats();
        assert_eq!(stats.document(&log),
            Stats {
                entries: 3,
                total_errors: 2,
                total_fatals: 0,
                average_latency: 393,
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
        assert_eq!(stats.document(&log),
            Stats {
                entries: 4,
                total_errors: 2,
                total_fatals: 1,
                average_latency: 420,
            });
    }
}
