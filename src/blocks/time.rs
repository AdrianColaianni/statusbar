use std::time::{Duration, SystemTime};
use chrono::{DateTime, Local, Timelike, Utc};

pub struct Time {
    output: String,
    last_run: SystemTime,
    frequency: Duration,
}

impl super::Block for Time {
    fn new() -> Self {
        let frequency = Duration::from_secs(1);
        let last_run = SystemTime::UNIX_EPOCH;
        let output = String::new();
        Self {
            output,
            last_run,
            frequency,
        }
    }

    fn frequency(&self) -> Duration {
        self.frequency
    }

    fn update(&mut self) -> bool {
        if self.last_run + self.frequency > SystemTime::now() {
            return false;
        }

        let local: DateTime<Local> = Local::now();
        let utc: DateTime<Utc> = Utc::now();
        let icon = match local.hour() % 12 {
            1 => "🕐",
            2 => "🕑",
            3 => "🕒",
            4 => "🕓",
            5 => "🕔",
            6 => "🕕",
            7 => "🕖",
            8 => "🕗",
            9 => "🕘",
            10 => "🕙",
            11 => "🕚",
            _ => "🕛",
        };
        self.output = format!(
            "{} {icon}{} - {:02}",
            local.format("%a %d %b"),
            local.format("%T"),
            utc.hour()
        );
        true
    }

    fn get_text(&self) -> String {
        self.output.clone()
    }
}
