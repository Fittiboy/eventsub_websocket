use std::net::TcpStream;
use std::sync::mpsc::{SendError, Sender};
use tungstenite::{connect, stream::MaybeTlsStream, WebSocket};
use url::{ParseError, Url};

use crate::handlers::*;
use crate::types::{MessageFields, Session, TwitchMessage};

pub use serde_json::from_str as parse_message;

pub mod handlers;
pub mod types;

pub type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
pub enum SessionErr {
    Parse(ParseError),
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

pub fn get_session() -> Result<Session, EventSubErr> {
    let (socket, _) = connect(Url::parse("wss://eventsub-beta.wss.twitch.tv/ws")?)?;
    Ok(Session::new(socket))
}

#[derive(Debug)]
pub enum EventSubErr {
    GeneralHandler(HandlerErr),
    Socket(tungstenite::Error),
    Session(SessionErr),
    Sending(SendError<String>),
}

impl std::fmt::Display for EventSubErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:#?}", self)
    }
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

impl From<SendError<String>> for EventSubErr {
    fn from(err: SendError<String>) -> Self {
        EventSubErr::Sending(err)
    }
}

pub fn event_handler(
    session: &mut Session,
    tx: Sender<String>,
) -> std::result::Result<(), EventSubErr> {
    loop {
        let msg = session.socket().read_message()?;
        let msg_raw = msg.to_text()?.to_owned();
        let msg: TwitchMessage = match serde_json::from_str(&msg_raw) {
            Ok(msg) => msg,
            Err(_) => continue,
        };

        tx.send(msg_raw)?;

        let message_id = msg.id();

        if session.handled().contains(&message_id) {
            println!("Duplicate message: {:#?}", msg);
            continue;
        }

        msg.handle(Some(session))?;

        session.handled().push(message_id.to_owned());
    }
}

#[cfg(test)]
mod tests {}
