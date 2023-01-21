use std::net::TcpStream;
use std::sync::mpsc::{SendError, Sender};
use thiserror::Error;
use tungstenite::{connect, stream::MaybeTlsStream, WebSocket};
use url::{ParseError, Url};

use crate::handlers::*;
use crate::types::{MessageFields, Session, TwitchMessage};

pub use serde_json::from_str as parse_message;

pub mod handlers;
pub mod types;

pub type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

#[derive(Error, Debug)]
pub enum SessionErr {
    #[error("error parsing url: {0}")]
    Parse(ParseError),
    #[error("connection error: {0}")]
    Connect(tungstenite::Error),
}

impl From<ParseError> for EventSubErr {
    fn from(err: ParseError) -> Self {
        EventSubErr::Session(SessionErr::Parse(err))
    }
}

impl From<tungstenite::Error> for SessionErr {
    fn from(err: tungstenite::Error) -> Self {
        SessionErr::Connect(err)
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

#[derive(Error, Debug)]
pub enum EventSubErr {
    #[error("general handler error: {0}")]
    GeneralHandler(HandlerErr),
    #[error("socket error: {0}")]
    Socket(tungstenite::Error),
    #[error("session error: {0}")]
    Session(SessionErr),
    #[error("error sending through channel: {0}")]
    Sending(SendError<TwitchMessage>),
}

impl From<EventSubErr> for String {
    fn from(err: EventSubErr) -> String {
        err.to_string()
    }
}

impl From<HandlerErr> for EventSubErr {
    fn from(err: HandlerErr) -> Self {
        EventSubErr::GeneralHandler(err)
    }
}

impl From<SessionErr> for EventSubErr {
    fn from(err: SessionErr) -> Self {
        EventSubErr::Session(err)
    }
}

impl From<tungstenite::Error> for EventSubErr {
    fn from(err: tungstenite::Error) -> Self {
        EventSubErr::Socket(err)
    }
}

impl From<SendError<TwitchMessage>> for EventSubErr {
    fn from(err: SendError<TwitchMessage>) -> Self {
        EventSubErr::Sending(err)
    }
}

pub fn event_handler(
    session: &mut Session,
    tx: Sender<TwitchMessage>,
) -> std::result::Result<(), EventSubErr> {
    loop {
        let msg = session.socket().read_message()?;
        let msg_raw = msg.to_text()?.to_owned();
        let msg: TwitchMessage = match serde_json::from_str(&msg_raw) {
            Ok(msg) => msg,
            Err(_) => continue,
        };

        let message_id = msg.id();

        if session.handled().contains(&message_id) {
            println!("Duplicate message: {:#?}", msg);
            continue;
        }

        msg.handle(Some(session), tx.clone())?;

        tx.send(msg)?;

        session.handled().push(message_id.to_owned());
    }
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
