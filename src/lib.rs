use std::sync::mpsc::Sender;
use tungstenite::connect;
use url::Url;

use crate::handlers::error::*;
use crate::types::{MessageId, Session, TwitchMessage};

pub use crate::handlers::error;
pub use serde_json::from_str as parse_message;

pub mod handlers;
pub mod types;

pub fn event_handler(
    session: &mut Session,
    tx: Sender<TwitchMessage>,
) -> std::result::Result<(), EventSubErr> {
    loop {
        let msg = session.socket.read_message()?;
        let msg_raw = msg.to_text()?.to_owned();
        let msg: TwitchMessage = match serde_json::from_str(&msg_raw) {
            Ok(msg) => msg,
            Err(_) => continue,
        };

        let message_id = msg.id();

        if session.handled.contains(&message_id) {
            println!("Duplicate message: {:#?}", msg);
            continue;
        }

        msg.handle(Some(session), tx.clone())?;

        tx.send(msg)?;

        session.handled.push(message_id.to_owned());
    }
}

pub fn get_session(url: Option<&str>) -> Result<Session, EventSubErr> {
    let to_parse;
    if let Some(url) = url {
        to_parse = url;
    } else {
        to_parse = "wss://eventsub-beta.wss.twitch.tv/ws";
    }
    let (socket, _) = connect(Url::parse(to_parse)?)?;
    Ok(Session::new(socket))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::thread;

    #[test]
    fn connect_to_mock() {
        get_session(Some("ws://localhost:8080/eventsub")).unwrap();
    }

    #[test]
    fn handle_welcome_message() {
        let (tx, rx): (Sender<TwitchMessage>, Receiver<TwitchMessage>) = mpsc::channel();
        let mut session = get_session(Some("ws://localhost:8080/eventsub")).unwrap();
        let _ =
            thread::Builder::new()
                .name("handler".into())
                .spawn(move || -> Result<(), String> {
                    event_handler(&mut session, tx).unwrap();
                    Ok(())
                });
        loop {
            let msg: TwitchMessage = rx.recv().map_err(|err| format!("{}", err)).unwrap();
            match msg {
                TwitchMessage::Welcome(_) => {
                    return ();
                }
                _ => {}
            }
        }
    }

    #[test]
    fn handle_reconnect_message() {
        let mut welcome_count = 0;
        let (tx, rx): (Sender<TwitchMessage>, Receiver<TwitchMessage>) = mpsc::channel();
        let mut session = get_session(Some("ws://localhost:8080/eventsub")).unwrap();
        let _ =
            thread::Builder::new()
                .name("handler".into())
                .spawn(move || -> Result<(), String> {
                    event_handler(&mut session, tx).unwrap();
                    Ok(())
                });
        loop {
            let msg: TwitchMessage = rx.recv().map_err(|err| format!("{}", err)).unwrap();
            match msg {
                TwitchMessage::Welcome(_) => {
                    welcome_count += 1;
                    if welcome_count > 1 {
                        return ();
                    }
                }
                _ => {}
            }
        }
    }
}
