use std::time::{Duration, SystemTime};

use pulsectl::controllers::{DeviceControl, SinkController, types::DevState};

pub struct Volume {
    output: String,
    last_run: SystemTime,
    frequency: Duration,
    sink: SinkController,
}

impl super::Block for Volume {
    fn new() -> Self {
        let frequency = Duration::from_millis(100);
        let last_run = SystemTime::UNIX_EPOCH;
        let output = String::new();
        let sink = SinkController::create().unwrap();
        Self {
            output,
            last_run,
            frequency,
            sink,
        }
    }

    fn frequency(&self) -> Duration {
        self.frequency
    }

    fn update(&mut self) -> bool {
        (|| -> Option<()> {
            if self.last_run + self.frequency > SystemTime::now() {
                return None;
            }

            // Get active device
            let dev = self
                .sink
                .list_devices()
                .ok()?
                .into_iter()
                .filter(|d| d.state == DevState::Running)
                .next()?;

            if dev.mute {
                self.output = String::from("🔇");
                return Some(());
            }

            let vol = dev.volume.get()[0].print();
            let vol = vol.trim();

            let icon = if vol < "30%" {
                "🔈"
            } else if vol < "60%" {
                "🔉"
            } else {
                "🔊"
            };
            self.output = format!("{icon}{vol}");
            self.last_run = SystemTime::now();

            Some(())
        })()
        .is_some()
    }

    fn get_text(&self) -> String {
        self.output.clone()
    }
}
