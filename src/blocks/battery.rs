use std::fs::File;
use std::io::Read;
use std::time::{Duration, SystemTime};

pub struct Battery {
    output: String,
    last_run: SystemTime,
    frequency: Duration,
}

impl super::Block for Battery {
    fn new() -> Self {
        let frequency = Duration::from_secs(30);
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

        (|| -> Result<(), std::io::Error> {
            // Status
            let mut status = File::open("/sys/class/power_supply/BAT1/status")?;
            let mut bat = "".to_string();
            status.read_to_string(&mut bat)?;
            let icon = match bat.as_str() {
                "Discharging\n" => "🔋",
                "Full\n" => "⚡",
                "Charging\n" => "🔌",
                "Not charging\n" => "🛑",
                _ => "♻️",
            };

            // Percent
            let mut status = File::open("/sys/class/power_supply/BAT1/capacity")?;
            bat.clear();
            status.read_to_string(&mut bat)?;
            let bat = bat.trim();

            // Low bat warning
            let warn = if icon == "🔋" && bat != "100" {
                if bat < "15" {
                    "‼️"
                } else if bat < "30" {
                    "❗"
                } else {
                    ""
                }
            } else {
                ""
            };
            self.output = format!("{icon}{warn}{bat}%");
            Ok(())
        })()
        .is_ok()
    }

    fn get_text(&self) -> String {
        self.output.clone()
    }
}
