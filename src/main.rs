#![allow(clippy::uninlined_format_args)]

use eventsub_websocket::event_handler;
use eventsub_websocket::types::TwitchMessage;
use std::sync::mpsc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();
    event_handler(None, tx)?;
    loop {
        let msg: TwitchMessage = rx.recv().map_err(|err| format!("{}", err))?;
        println!("Handling message locally: {:#?}", msg);
    }
}
