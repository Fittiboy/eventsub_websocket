#![allow(clippy::uninlined_format_args)]

use eventsub_websocket::types::TwitchMessage;
use eventsub_websocket::{event_handler, get_default_url};
use std::sync::mpsc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();
    let default_url = get_default_url()?;
    event_handler(default_url, tx)?;
    loop {
        let msg: TwitchMessage = rx.recv().map_err(|err| format!("{}", err))?;
        println!("Handling message locally: {:#?}", msg);
    }
}
