use crate::types::{Session, TwitchMessage, Welcome};

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
            _ => {}
        }
    }
}

impl Welcome {
    fn handle(&self, session: Option<&mut Session>) {
        if let Some(session) = session {
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
