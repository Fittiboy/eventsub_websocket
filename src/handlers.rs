use crate::types::{
    Keepalive, Notification, Reconnect, Revocation, Session, TwitchMessage, Welcome,
};

pub trait Handler
where
    Self: std::fmt::Debug,
{
    fn handle(&self, session: Option<&mut Session>) {
        match session {
            Some(session) => println!("Session: {:#?}", session),
            None => println!("Message received: {:#?}", self),
        }
    }
}

impl Handler for TwitchMessage {
    fn handle(&self, session: Option<&mut Session>) {
        match self {
            TwitchMessage::Welcome(msg) => msg.handle(session),
            TwitchMessage::Keepalive(msg) => msg.handle(None),
            TwitchMessage::Notification(msg) => msg.handle(None),
            TwitchMessage::Revocation(msg) => msg.handle(None),
            TwitchMessage::Reconnect(msg) => msg.handle(None),
        }
    }
}

impl Handler for Welcome {
    fn handle(&self, session: Option<&mut Session>) {
        if let Some(session) = session {
            println!("{}", serd_string);
            session.set_id(self.payload().session().id().to_string());
            println!("Welcome received: {:#?}", self);
            if let Some(time) = &self.payload().session().keepalive() {
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
