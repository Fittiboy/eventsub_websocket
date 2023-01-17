use serde::{Deserialize, Serialize};
use serde_json::Number;
use serde_json::Value;
use std::net::TcpStream;
use std::time::Duration;
use tungstenite::{stream::MaybeTlsStream, WebSocket};

pub type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

pub trait MessageFields {
    fn get_id(&self) -> String;
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

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn handled(&mut self) -> &mut Vec<String> {
        &mut self.handled
    }

    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }

    pub fn set_keepalive(&mut self, keepalive: u64) {
        match self.socket().get_mut() {
            MaybeTlsStream::NativeTls(stream) => {
                let stream = stream.get_mut();
                stream
                    // Allow a short grace period by adding one second to the reported keepalive
                    // timeout
                    .set_read_timeout(Some(Duration::from_secs(keepalive + 1)))
                    .unwrap();
            }
            _ => unreachable!(),
        }
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
pub struct SessionData {
    id: String,
    status: String,
    connected_at: String,
    keepalive_timeout_seconds: Option<Number>,
    reconnect_url: Option<String>,
}

impl SessionData {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn keepalive(&self) -> &Option<Number> {
        &self.keepalive_timeout_seconds
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SessionPayload {
    session: SessionData,
}

impl SessionPayload {
    pub fn session(&self) -> &SessionData {
        &self.session
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Welcome {
    metadata: GenericMetadata,
    payload: SessionPayload,
}

impl Welcome {
    pub fn payload(&self) -> &SessionPayload {
        &self.payload
    }
}

impl MessageFields for Welcome {
    fn get_id(&self) -> String {
        self.metadata.message_id.clone()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Keepalive {
    metadata: GenericMetadata,
    payload: Value,
}

impl Keepalive {
    pub fn payload(&self) -> &Value {
        &self.payload
    }
}

impl MessageFields for Keepalive {
    fn get_id(&self) -> String {
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

impl SubscriptionPayload {
    pub fn id(&self) -> &str {
        &self.id
    }
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
    fn get_id(&self) -> String {
        self.metadata.message_id.clone()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Reconnect {
    metadata: GenericMetadata,
    payload: SessionPayload,
}

impl Reconnect {
    pub fn payload(&self) -> &SessionPayload {
        &self.payload
    }
}

impl MessageFields for Reconnect {
    fn get_id(&self) -> String {
        self.metadata.message_id.clone()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Revocation {
    metadata: SubscriptionMetadata,
    payload: SubscriptionPayload,
}

impl Revocation {
    pub fn payload(&self) -> &SubscriptionPayload {
        &self.payload
    }
}

impl MessageFields for Revocation {
    fn get_id(&self) -> String {
        self.metadata.message_id.clone()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum TwitchMessage {
    Welcome(Welcome),
    Keepalive(Keepalive),
    Notification(Notification),
    Reconnect(Reconnect),
    Revocation(Revocation),
}

impl MessageFields for TwitchMessage {
    fn get_id(&self) -> String {
        match self {
            Self::Welcome(msg) => msg.get_id(),
            Self::Keepalive(msg) => msg.get_id(),
            Self::Notification(msg) => msg.get_id(),
            Self::Reconnect(msg) => msg.get_id(),
            Self::Revocation(msg) => msg.get_id(),
        }
    }
}
