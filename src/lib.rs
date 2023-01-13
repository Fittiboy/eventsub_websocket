use json::{self, JsonValue};
use std::net::TcpStream;
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;

pub type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

pub fn get_session() -> Socket {
    let (socket, _) = connect(Url::parse("wss://eventsub-beta.wss.twitch.tv/ws").unwrap())
        .expect("Cannot connect");

    socket

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

fn format_message(msg: Message) -> JsonValue {
    let data = msg.to_text().unwrap();
    let data = format!("[{}]", data);
    json::parse(&data).unwrap()
}

pub fn handle_welcome(socket: &mut Socket) -> JsonValue {
    loop {
        let msg = socket.read_message().expect("Error reading message");
        let parsed = format_message(msg);
        let id = &parsed[0]["payload"]["session"]["id"];
        if !id.is_null() {
            return parsed;
        }
    }
}

pub fn event_handler(socket: &mut Socket) {
    loop {
        let msg = socket.read_message().expect("Error reading message");
        let parsed = format_message(msg);
    }
}

#[cfg(test)]
mod tests {}
