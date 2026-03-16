use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEvent {
    ts: DateTime<Local>,
    level: String,
    service: String,
    latency: usize,
    endpoint: String,
}

#[derive(Debug)]
pub struct Stats {
    entries: usize,
    errors: usize,
    fatals: usize,
    average_latency: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
