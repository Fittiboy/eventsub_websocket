#![allow(clippy::uninlined_format_args)]

use eventsub_websocket::types::TwitchMessage;
use eventsub_websocket::{event_handler, get_session};
use std::sync::mpsc;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();
    let mut session = get_session(None)?;
    let _ = thread::Builder::new()
        .name("handler".into())
        .spawn(move || -> Result<(), String> {
            event_handler(&mut session, tx)?;
            Ok(())
        });
    loop {
        let msg: TwitchMessage = rx.recv().map_err(|err| format!("{}", err))?;
        println!("Handling message locally: {:#?}", msg);
    }
}
