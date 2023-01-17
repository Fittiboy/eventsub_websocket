use serde_json::{Result, Value};
use std::net::TcpStream;
use tungstenite::{connect, stream::MaybeTlsStream, WebSocket};
use url::Url;

use crate::handlers::*;
use crate::types::{
    Keepalive, MessageFields, Notification, Reconnect, Revocation, Session, TwitchMessage, Welcome,
};

pub mod handlers;
pub mod types;

pub type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

pub fn get_session() -> Session {
    let (socket, _) = connect(Url::parse("wss://eventsub-beta.wss.twitch.tv/ws").unwrap())
        .expect("Cannot connect");

    Session::new(socket)
}

fn parse_message(msg: &str) -> Result<TwitchMessage> {
    let msg: TwitchMessage = serde_json::from_str(msg)?;
    return Ok(msg);
}

pub fn event_handler(session: &mut Session) -> std::result::Result<(), Box<dyn std::error::Error>> {
    loop {
        let msg = session
            .socket()
            .read_message()
            .expect("Connection closed due to timeout!");
        let msg = msg.to_text()?.to_owned();
        let parsed: Value = match serde_json::from_str(&msg) {
            Ok(value) => value,
            Err(_) => continue,
        };

        let msg = match parse_message(&msg) {
            Ok(msg) => msg,
            Err(_) => continue,
        };

        let message_id = msg.get_id();

        if session.handled().contains(&message_id) {
            println!("Duplicate message: {:#?}", parsed);
            continue;
        }

        msg.handle(Some(session));

        session.handled().push(message_id.to_owned());
    }
}

#[cfg(test)]
mod tests {}
