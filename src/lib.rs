use serde_json::{Result, Value};
use std::net::TcpStream;
use tungstenite::{connect, stream::MaybeTlsStream, WebSocket};
use url::Url;

use crate::handlers::*;
use crate::types::{TwitchMessage::*, *};

mod handlers;
mod types;

pub type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

pub fn get_session() -> Session {
    let (socket, _) = connect(Url::parse("wss://eventsub-beta.wss.twitch.tv/ws").unwrap())
        .expect("Cannot connect");

    Session::new(socket)

    // let _ = socket.write_message(Message::Text(
    //     r#"{
    //     "action": "authenticate",
    //     "data": {
    //         "key_id": "API-KEY",
    //         "secret_key": "SECRET-KEY"
    //     }
    // }"#
    //     .into(),
    // ));
}

fn parse_message(msg: &str) -> Result<TwitchMessage> {
    let parsed: Value = serde_json::from_str(msg)?;
    let msg_type = &parsed["metadata"]["message_type"];

    if msg_type == "session_welcome" {
        let welcome: Welcome = serde_json::from_str(msg)?;
        return Ok(WelcomeMessage(welcome));
    } else if msg_type == "session_keepalive" {
        let welcome: Keepalive = serde_json::from_str(msg)?;
        return Ok(KeepaliveMessage(welcome));
    } else if msg_type == "notification" {
        let welcome: Notification = serde_json::from_str(msg)?;
        return Ok(NotificationMessage(welcome));
    } else if msg_type == "session_reconnect" {
        let welcome: Reconnect = serde_json::from_str(msg)?;
        return Ok(ReconnectMessage(welcome));
    } else if msg_type == "revocation" {
        let welcome: Revocation = serde_json::from_str(msg)?;
        return Ok(RevocationMessage(welcome));
    } else {
        panic!("This match arm should be unreachable!");
    };
}

pub fn event_handler(session: &mut Session) -> std::result::Result<(), Box<dyn std::error::Error>> {
    loop {
        let msg = session.socket().read_message()?;
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
