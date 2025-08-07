use chrono::{DateTime, Local, Timelike, Utc};
use pulsectl::controllers::DeviceControl;
use pulsectl::controllers::SinkController;
use pulsectl::controllers::types::DevState;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the X11 server
    let (conn, screen_num) = RustConnection::connect(None)?;

    // Get root window of the current screen
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let mut blocks = vec![
        Block::new(Duration::from_millis(100), Box::new(volume)),
        Block::new(Duration::from_secs(20), Box::new(battery)),
        Block::new(Duration::from_secs(1), Box::new(time)),
        Block::new(Duration::from_secs(10), Box::new(internet)),
    ];

    let sleep_time = blocks
        .iter()
        .map(|b| b.frequency)
        .min()
        .unwrap_or(Duration::from_millis(10));

    let mut nothing_happened;
    loop {
        nothing_happened = true;
        for block in blocks.iter_mut() {
            if block.should_update() {
                block.update();
                nothing_happened = false;
            }
        }

        if nothing_happened {
            sleep(sleep_time);
            continue;
        }

        let bar: Vec<String> = blocks.iter().map(|b| b.output.clone()).collect();
        let bar = bar.join("|");
        // println!("{}", bar);

        // Set the WM_NAME property to the input string
        conn.change_property8(
            PropMode::REPLACE,
            root,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            bar.as_bytes(),
        )?;

        // Flush the request
        conn.flush()?;

        sleep(sleep_time);
    }
}

struct Block {
    output: String,
    last_run: SystemTime,
    frequency: Duration,
    command: Box<dyn Fn() -> String + Send + Sync>,
}

impl Block {
    fn new(frequency: Duration, command: Box<dyn Fn() -> String + Send + Sync>) -> Self {
        let output = command();
        let last_run = SystemTime::now();
        Block {
            output,
            last_run,
            frequency,
            command,
        }
    }

    fn update(&mut self) {
        self.output = (self.command)();
        self.last_run = SystemTime::now();
    }

    fn should_update(&self) -> bool {
        self.last_run + self.frequency <= SystemTime::now()
    }
}

fn run_cmd(cmd: &str, args: &[&str]) -> String {
    let res = Command::new(cmd).args(args).output().ok();
    let res = res.map(|r| r.stdout).unwrap_or(Vec::new());

    str::from_utf8(&res).unwrap_or("None").trim().to_string()
}

fn volume() -> String {
    // Setup pulse
    let dev = &SinkController::create().unwrap().list_devices().unwrap();

    // Get active device
    let dev = dev
        .into_iter()
        .filter(|d| d.state == DevState::Running)
        .next()
        .unwrap();

    if dev.mute {
        return String::from("🔇");
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
    format!("{icon}{vol}")
}

fn battery() -> String {
    (|| -> Result<String, std::io::Error> {
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
        let warn = if icon == "🔋" {
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
        Ok(format!("{icon}{warn}{bat}%"))
    })()
    .unwrap_or_default()
}

fn time() -> String {
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
    format!(
        "{} {icon}{} - {:02}",
        local.format("%a %d %b"),
        local.format("%T"),
        utc.hour()
    )
}

fn internet() -> String {
    let strength = run_cmd("/usr/bin/nmcli", &["-f", "IN-USE,SIGNAL", "device", "wifi"]);
    let strength = strength.split_once("*").unwrap().1;
    let strength = strength.trim().split_once(" ").unwrap().0;
    format!("📶{strength}%")
}
