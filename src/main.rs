mod blocks;
use blocks::*;

use std::thread::sleep;
use std::time::Duration;
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

    let mut blocks: Vec<Box<dyn blocks::Block>> = vec![
        Box::new(volume::Volume::new()),
        Box::new(battery::Battery::new()),
        Box::new(time::Time::new()),
        Box::new(internet::Internet::new()),
    ];

    let sleep_time = blocks
        .iter()
        .map(|b| b.frequency())
        .min()
        .unwrap_or(Duration::from_millis(10));

    let mut nothing_happened;
    loop {
        nothing_happened = true;
        for block in blocks.iter_mut() {
            if block.update() {
                nothing_happened = false;
            }
        }

        if nothing_happened {
            sleep(sleep_time);
            continue;
        }

        let bar: Vec<String> = blocks.iter().map(|b| b.get_text()).collect();
        let mut bar = bar.join("|");
        bar.push(' ');

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
