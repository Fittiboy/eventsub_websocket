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
    let msg_type = &parsed["message_type"];

    if msg_type == "session_welcome" {
        let welcome: Welcome = serde_json::from_str(msg)?;
        return Ok(WelcomeMessage(welcome));
    } else if msg_type == "session_keepalive" {
        let welcome: Keepalive = serde_json::from_str(msg)?;
        return Ok(KeepaliveMessage(welcome));
    } else if msg_type == "notification" {
        let welcome: Notification = serde_json::from_str(msg)?;
        return Ok(NotificationMessage(welcome));
    } else {
        let welcome: Other = serde_json::from_str(msg)?;
        return Ok(OtherMessage(welcome));
    };
}

pub fn event_handler(session: &mut Session) -> Result<()> {
    loop {
        let msg = session
            .socket()
            .read_message()
            .expect("Error reading message");
        let data = msg.to_text().unwrap();
        let parsed: Value = match serde_json::from_str(&data) {
            Ok(value) => value,
            Err(_) => continue,
        };

        let msg = match parse_message(data) {
            Ok(msg) => msg,
            Err(_) => continue,
        };

        let message_id = msg.get_id();

        if session.handled().contains(&message_id) {
            println!("Duplicate message: {:#?}", parsed);
            continue;
        }

        match msg {
            WelcomeMessage(msg) => handle_welcome(msg, session),
            KeepaliveMessage(msg) => handle_keepalive(msg),
            NotificationMessage(msg) => handle_notification(msg),
            OtherMessage(msg) => handle_other(msg),
        };

        session.handled().push(message_id.to_owned());
        return Ok(());
    }
}

#[cfg(test)]
mod tests {}
