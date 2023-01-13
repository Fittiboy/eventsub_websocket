use json::{self, JsonValue};
use std::net::TcpStream;
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;

pub struct Session {
    socket: Socket,
    id: String,
    handled: Vec<String>,
}

impl Session {
    pub fn new(socket: Socket) -> Session {
        Session {
            socket,
            id: String::new(),
            handled: vec![],
        }
    }
}

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

fn format_message(msg: Message) -> JsonValue {
    let data = msg.to_text().unwrap();
    let data = format!("[{}]", data);
    json::parse(&data).unwrap()[0].to_owned()
}

fn handle_welcome(msg: JsonValue, session: &mut Session) {
    println!("Welcome!");
    let id = &msg["payload"]["session"]["id"];
    if !id.is_null() {
        session.id = String::from(id.to_string());
    };
}

fn handle_keepalive(msg: JsonValue) {
    println!("Keepalive received: {}", msg);
}

fn handle_notification(msg: JsonValue) {
    println!("Notifiaction received: {}", msg);
}

pub fn event_handler(session: &mut Session) {
    loop {
        let msg = session
            .socket
            .read_message()
            .expect("Error reading message");
        let parsed = format_message(msg);

        if parsed.is_null() {
            println!("Empty message: {}", parsed);
            continue;
        }

        let metadata = &parsed["metadata"];
        let msg_id = String::from(&metadata["message_id"].to_string());

        if session.handled.contains(&msg_id) {
            println!("Duplicate message: {}", parsed);
            continue;
        }

        let msg_type = &metadata["message_type"];

        if msg_type == "session_welcome" {
            handle_welcome(parsed, session);
        } else if msg_type == "session_keepalive" {
            handle_keepalive(parsed);
        } else if msg_type == "notification" {
            handle_notification(parsed);
        } else {
            println!("Received message: {}", parsed);
        };

        session.handled.push(msg_id);
    }
}

#[cfg(test)]
mod tests {}
