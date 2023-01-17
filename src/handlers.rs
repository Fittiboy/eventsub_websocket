use crate::types::{Session, TwitchMessage, Welcome};
use std::io;

#[derive(Debug)]
pub enum HandlerErr {
    Welcome(WelcomeHandlerErr),
}

impl From<WelcomeHandlerErr> for HandlerErr {
    fn from(err: WelcomeHandlerErr) -> Self {
        HandlerErr::Welcome(err)
    }
}

pub trait Handler
where
    Self: std::fmt::Debug,
{
    fn handle(&self, session: Option<&mut Session>) -> Result<(), HandlerErr>;
}

impl Handler for TwitchMessage {
    fn handle(&self, session: Option<&mut Session>) -> Result<(), HandlerErr> {
        match self {
            TwitchMessage::Welcome(msg) => Ok(msg.handle(session)?),
            TwitchMessage::Reconnect(msg) => todo!("{:#?}", msg),
            _ => Ok(()),
        }
    }
}

#[derive(Debug)]
pub enum WelcomeHandlerErr {
    NoKeepalive(String),
    InvalidKeepalive(String),
    NoSession(String),
    CannotSetKeepalive(io::Error),
}

impl From<io::Error> for WelcomeHandlerErr {
    fn from(err: io::Error) -> Self {
        WelcomeHandlerErr::CannotSetKeepalive(err)
    }
}

impl Welcome {
    fn handle(&self, session: Option<&mut Session>) -> Result<(), WelcomeHandlerErr> {
        if let Some(session) = session {
            session.set_id(self.session_id().to_string());
            if let Some(time) = &self.keepalive() {
                let keepalive = time.as_u64();
                match keepalive {
                    Some(time) => session.set_keepalive(time)?,
                    None => {
                        return Err(WelcomeHandlerErr::InvalidKeepalive(format!(
                            "invalid keepalive time received: {}",
                            time
                        )))
                    }
                }
            } else {
                return Err(WelcomeHandlerErr::NoKeepalive(
                    "Twitch did not return a keepalive time!".to_string(),
                ));
            };
        } else {
            return Err(WelcomeHandlerErr::NoSession(
                "Welcome handler needs to be called with valid session".to_string(),
            ));
        };
        Ok(())
    }
}
