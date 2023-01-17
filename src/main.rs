use eventsub_websocket::types::TwitchMessage;
use eventsub_websocket::{event_handler, get_session};
use std::sync::mpsc;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();
    let mut session = get_session();
    let _ = thread::Builder::new()
        .name("handler".into())
        .spawn(move || -> Result<(), String> {
            event_handler(&mut session, tx).map_err(|err| err.to_string())?;
            Ok(())
        });
    loop {
        let msg = rx.recv().unwrap();
        let msg: TwitchMessage = eventsub_websocket::parse_message(&msg).unwrap();
        println!("Handling message locally: {:#?}", msg);
    }
}
