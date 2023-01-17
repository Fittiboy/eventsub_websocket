use crate::types::{Session, TwitchMessage, Welcome};

pub enum HandlerErr {
    Welcome(WelcomeHandlerErr),
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
            TwitchMessage::Welcome(msg) => {
                msg.handle(session).map_err(|err| HandlerErr::Welcome(err))
            }
            TwitchMessage::Reconnect(msg) => todo!("{:#?}", msg),
            _ => Ok(()),
        }
    }
}

pub enum WelcomeHandlerErr {
    NoKeepalive(String),
    InvalidKeepalive(String),
    NoSession(String),
}

impl Welcome {
    fn handle(&self, session: Option<&mut Session>) -> Result<(), WelcomeHandlerErr> {
        if let Some(session) = session {
            session.set_id(self.session_id().to_string());
            if let Some(time) = &self.keepalive() {
                let keepalive = time.as_u64();
                match keepalive {
                    Some(time) => session.set_keepalive(time),
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
