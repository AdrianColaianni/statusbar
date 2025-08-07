use std::time::{Duration, SystemTime};

use network_manager::{ConnectionSettings, NetworkManager};

pub struct Internet {
    output: String,
    last_run: SystemTime,
    frequency: Duration,
    nm: NetworkManager,
}

impl super::Block for Internet {
    fn new() -> Self {
        let frequency = Duration::from_secs(2);
        let last_run = SystemTime::UNIX_EPOCH;
        let output = String::new();
        let nm = NetworkManager::new();
        Self {
            output,
            last_run,
            frequency,
            nm,
        }
    }

    fn frequency(&self) -> Duration {
        self.frequency
    }

    fn update(&mut self) -> bool {
        if self.last_run + self.frequency > SystemTime::now() {
            return false;
        }

        (|| -> Option<()> {
            let con: Vec<ConnectionSettings> = self
                .nm
                .get_active_connections()
                .ok()?
                .into_iter()
                .map(|c| c.settings().clone())
                .collect();

            if con.len() == 1 {
                // Only loopback dev
                self.output = String::from("📡");
            }

            self.output.clear();

            // Wireless
            if con.iter().any(|c| c.kind == "802-11-wireless") {
                self.output.push('📶');
            }

            // VPN
            if con.iter().any(|c| c.kind == "wireguard") {
                self.output.push('🔒');
            }

            Some(())
        })()
        .is_some()
    }

    fn get_text(&self) -> String {
        self.output.clone()
    }
}
