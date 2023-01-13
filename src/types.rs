use serde::{Deserialize, Serialize};
use serde_json::Number;
use serde_json::Value;
use std::net::TcpStream;
use tungstenite::{stream::MaybeTlsStream, WebSocket};

pub type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

pub struct Session {
    pub socket: Socket,
    pub id: String,
    pub handled: Vec<String>,
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

#[derive(Deserialize, Serialize, Debug)]
pub struct GenericMetadata {
    message_id: String,
    message_type: String,
    message_timestamp: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SessionData {
    pub id: String,
    status: String,
    connected_at: String,
    keepalive_timeout_seconds: Number,
    reconnect_url: Option<Value>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WelcomePayload {
    pub session: SessionData,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Welcome {
    metadata: GenericMetadata,
    pub payload: WelcomePayload,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Keepalive {
    metadata: GenericMetadata,
    pub payload: Value,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SubscriptionMetadata {
    message_id: String,
    message_type: String,
    message_timestamp: String,
    subscription_type: String,
    subscription_version: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SubscriptionPayload {
    pub id: String,
    status: String,
    r#type: String,
    version: String,
    cost: Number,
    condition: Value,
    transport: Value,
    created_at: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NotificationPayload {
    subscription: SubscriptionPayload,
    event: Value,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Notification {
    metadata: SubscriptionMetadata,
    pub payload: NotificationPayload,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Other {
    metadata: GenericMetadata,
    pub payload: Value,
}

#[derive(Debug)]
pub enum TwitchMessage {
    WelcomeMessage(Welcome),
    KeepaliveMessage(Keepalive),
    NotificationMessage(Notification),
    OtherMessage(Other),
}
