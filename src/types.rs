use crate::error::KeepaliveErr;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use serde_json::Value;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use tungstenite::{stream::MaybeTlsStream, WebSocket};

#[derive(Debug)]
/// The connection to the EventSub server with Twitch, which contains a socket, a session ID, and
/// the vector of handled messages (to avoid handling duplicates).
pub struct Session {
    /// The socket which is connected to Twitch's EventSub WebSocket server.
    pub socket: Arc<Mutex<Socket>>,
    /// The session ID Twitch returns with the `Welcome` message. Initially empty String.
    pub id: String,
    /// The `handled` vector contains the message IDs of those messages which have already been
    /// handled, to avoid taking action twice when Twitch repeats their notification.
    pub handled: Vec<String>,
}

pub struct EventResult {
    pub listener: JoinHandle<Result<(), String>>,
    pub socket: Arc<Mutex<crate::types::Socket>>,
}

/// This layered type is [`tungstenite`](https://crates.io/crates/tungstenite)'s WebSocket connection.
pub type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum TwitchMessage {
    Notification(Notification),
    Welcome(Welcome),
    Reconnect(Reconnect),
    Revocation(Revocation),
    Keepalive(Keepalive),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Notification {
    pub metadata: SubscriptionMetadata,
    pub payload: NotificationPayload,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Welcome {
    pub metadata: GenericMetadata,
    pub payload: WelcomeSessionPayload,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Reconnect {
    pub metadata: GenericMetadata,
    pub payload: ReconnectSessionPayload,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Keepalive {
    pub metadata: GenericMetadata,
    pub payload: Value,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Revocation {
    pub metadata: SubscriptionMetadata,
    pub payload: SubscriptionPayload,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GenericMetadata {
    pub message_id: String,
    pub message_type: String,
    pub message_timestamp: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WelcomeSessionData {
    pub id: String,
    pub status: String,
    pub connected_at: String,
    pub keepalive_timeout_seconds: Number,
    pub reconnect_url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReconnectSessionData {
    pub id: String,
    pub status: String,
    pub connected_at: String,
    pub keepalive_timeout_seconds: Option<Number>,
    pub reconnect_url: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WelcomeSessionPayload {
    pub session: WelcomeSessionData,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReconnectSessionPayload {
    pub session: ReconnectSessionData,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SubscriptionMetadata {
    pub message_id: String,
    pub message_type: String,
    pub message_timestamp: String,
    pub subscription_type: String,
    pub subscription_version: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SubscriptionPayload {
    pub id: String,
    pub status: String,
    pub r#type: String,
    pub version: String,
    pub cost: Number,
    pub condition: Value,
    pub transport: Value,
    pub created_at: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NotificationPayload {
    pub subscription: SubscriptionPayload,
    pub event: Value,
}

impl Session {
    pub fn new(socket: Arc<Mutex<Socket>>) -> Session {
        Session {
            socket,
            id: String::new(),
            handled: vec![],
        }
    }

    /// Sets the timeout on the `Session`'s contained `Socket` to match the keepalive time returned
    /// by Twitch in a `Welcome` message. Adds an extra second as a grace period.
    pub fn set_keepalive(&mut self, keepalive: u64) -> Result<(), KeepaliveErr> {
        let mut binding = self.socket.lock()?;
        let stream = match binding.get_mut() {
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

impl TwitchMessage {
    /// Return a clone of the message ID
    pub fn id(&self) -> String {
        match self {
            Self::Welcome(msg) => msg.metadata.message_id.clone(),
            Self::Keepalive(msg) => msg.metadata.message_id.clone(),
            Self::Notification(msg) => msg.metadata.message_id.clone(),
            Self::Reconnect(msg) => msg.metadata.message_id.clone(),
            Self::Revocation(msg) => msg.metadata.message_id.clone(),
        }
    }
}
