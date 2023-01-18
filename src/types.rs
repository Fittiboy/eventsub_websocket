use serde::{Deserialize, Serialize};
use serde_json::Number;
use serde_json::Value;
use std::net::TcpStream;
use std::time::Duration;
use tungstenite::{stream::MaybeTlsStream, WebSocket};

pub type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

pub trait MessageFields {
    fn id(&self) -> String;
}

#[derive(Debug)]
pub struct Session {
    socket: Socket,
    id: String,
    handled: Vec<String>,
}

impl Session {
    pub fn socket(&mut self) -> &mut Socket {
        &mut self.socket
    }

    pub fn handled(&mut self) -> &mut Vec<String> {
        &mut self.handled
    }

    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }

    pub fn set_keepalive(&mut self, keepalive: u64) -> Result<(), std::io::Error> {
        let stream = match self.socket().get_mut() {
            MaybeTlsStream::NativeTls(stream) => stream.get_mut(),
            MaybeTlsStream::Plain(stream) => stream,
            _ => unreachable!("Stream has to always be either TLS or plain"),
        };
        stream
            // Allow a short grace period by adding one second to the reported keepalive
            // timeout
            .set_read_timeout(Some(Duration::from_secs(keepalive + 1)))?;
        Ok(())
    }
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
pub struct WelcomeSessionData {
    id: String,
    status: String,
    connected_at: String,
    keepalive_timeout_seconds: Number,
    reconnect_url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReconnectSessionData {
    id: String,
    status: String,
    connected_at: String,
    keepalive_timeout_seconds: Option<Number>,
    reconnect_url: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WelcomeSessionPayload {
    session: WelcomeSessionData,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReconnectSessionPayload {
    session: ReconnectSessionData,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Welcome {
    metadata: GenericMetadata,
    payload: WelcomeSessionPayload,
}

impl Welcome {
    pub fn session_id(&self) -> &str {
        &self.payload.session.id
    }

    pub fn keepalive(&self) -> &Number {
        &self.payload.session.keepalive_timeout_seconds
    }
}

impl MessageFields for Welcome {
    fn id(&self) -> String {
        self.metadata.message_id.clone()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Keepalive {
    metadata: GenericMetadata,
    payload: Value,
}

impl MessageFields for Keepalive {
    fn id(&self) -> String {
        self.metadata.message_id.clone()
    }
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
    id: String,
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
    payload: NotificationPayload,
}

impl Notification {
    pub fn payload(&self) -> &NotificationPayload {
        &self.payload
    }
}

impl MessageFields for Notification {
    fn id(&self) -> String {
        self.metadata.message_id.clone()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Reconnect {
    metadata: GenericMetadata,
    payload: ReconnectSessionPayload,
}

impl Reconnect {
    pub fn reconnect_url(&self) -> &String {
        &self.payload.session.reconnect_url
    }
}

impl MessageFields for Reconnect {
    fn id(&self) -> String {
        self.metadata.message_id.clone()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Revocation {
    metadata: SubscriptionMetadata,
    payload: SubscriptionPayload,
}

impl MessageFields for Revocation {
    fn id(&self) -> String {
        self.metadata.message_id.clone()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum TwitchMessage {
    Notification(Notification),
    Welcome(Welcome),
    Reconnect(Reconnect),
    Revocation(Revocation),
    Keepalive(Keepalive),
}

impl MessageFields for TwitchMessage {
    fn id(&self) -> String {
        match self {
            Self::Welcome(msg) => msg.id(),
            Self::Keepalive(msg) => msg.id(),
            Self::Notification(msg) => msg.id(),
            Self::Reconnect(msg) => msg.id(),
            Self::Revocation(msg) => msg.id(),
        }
    }
}
