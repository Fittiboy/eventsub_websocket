use crate::types::{TwitchMessage::*, *};

pub trait Handler
where
    Self: std::fmt::Debug,
{
    fn handle(&self, session: Option<&mut Session>) {
        match session {
            Some(session) => println!("Session: {:#?}", session),
            None => {}
        }
        println!("Message received: {:#?}", self)
    }
}

impl Handler for TwitchMessage {
    fn handle(&self, session: Option<&mut Session>) {
        match self {
            WelcomeMessage(msg) => msg.handle(session),
            KeepaliveMessage(msg) => msg.handle(None),
            NotificationMessage(msg) => msg.handle(None),
            RevocationMessage(msg) => msg.handle(None),
            ReconnectMessage(msg) => msg.handle(None),
        }
    }
}

impl Handler for Welcome {
    fn handle(&self, session: Option<&mut Session>) {
        if let Some(session) = session {
            session.set_id(self.payload.session.id.to_owned());
            println!("Welcome received: {:#?}", self);
            if let Some(time) = &self.payload.session.keepalive_timeout_seconds {
                session.set_keepalive(time.as_u64().unwrap());
            } else {
                panic!("Twitch did not return a keepalive time!");
            };
        } else {
            panic!("Welcome message needs session!");
        };
    }
}
impl Handler for Keepalive {}
impl Handler for Notification {}
impl Handler for Revocation {}
impl Handler for Reconnect {}
