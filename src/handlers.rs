use crate::types::{MessageFields, Reconnect, Session, TwitchMessage, Welcome};
use std::io;
use std::sync::mpsc::{SendError, Sender};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HandlerErr {
    #[error("error handling welcome message: {0}")]
    Welcome(WelcomeHandlerErr),
    #[error("error handling erconnect message: {0}")]
    Reconnect(ReconnectHandlerErr),
}

impl From<WelcomeHandlerErr> for HandlerErr {
    fn from(err: WelcomeHandlerErr) -> Self {
        HandlerErr::Welcome(err)
    }
}

impl From<ReconnectHandlerErr> for HandlerErr {
    fn from(err: ReconnectHandlerErr) -> Self {
        HandlerErr::Reconnect(err)
    }
}

pub trait Handler
where
    Self: std::fmt::Debug,
{
    fn handle(
        &self,
        session: Option<&mut Session>,
        tx: Sender<TwitchMessage>,
    ) -> Result<(), HandlerErr>;
}

impl Handler for TwitchMessage {
    fn handle(
        &self,
        session: Option<&mut Session>,
        tx: Sender<TwitchMessage>,
    ) -> Result<(), HandlerErr> {
        match self {
            TwitchMessage::Welcome(msg) => Ok(msg.handle(session)?),
            TwitchMessage::Reconnect(msg) => Ok(msg.handle(session, tx)?),
            _ => Ok(()),
        }
    }
}

#[derive(Error, Debug)]
pub enum WelcomeHandlerErr {
    #[error("Twitch did not return a keepalive: {0}")]
    NoKeepalive(String),
    #[error("Twitch returned an invalid keepalive: {0}")]
    InvalidKeepalive(String),
    #[error("no session was provided: {0}")]
    NoSession(String),
    #[error("error when setting keepalive: {0}")]
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
            let keepalive = self.keepalive().as_u64();
            match keepalive {
                Some(time) => session.set_keepalive(time)?,
                None => {
                    return Err(WelcomeHandlerErr::InvalidKeepalive(format!(
                        "invalid keepalive time received: {:#?}",
                        keepalive
                    )))
                }
            }
        } else {
            return Err(WelcomeHandlerErr::NoSession(
                "Welcome handler needs to be called with valid session".to_string(),
            ));
        };
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ReconnectHandlerErr {
    #[error("session error while reconnecting: {0}")]
    Session(String),
    #[error("general error while reconnecting: {0}")]
    Handler(String),
    #[error("connection error while reconnecting: {0}")]
    Connection(tungstenite::Error),
}

impl From<tungstenite::Error> for ReconnectHandlerErr {
    fn from(err: tungstenite::Error) -> ReconnectHandlerErr {
        ReconnectHandlerErr::Connection(err)
    }
}

impl From<HandlerErr> for ReconnectHandlerErr {
    fn from(err: HandlerErr) -> ReconnectHandlerErr {
        ReconnectHandlerErr::Handler(err.to_string())
    }
}

impl From<SendError<TwitchMessage>> for ReconnectHandlerErr {
    fn from(err: SendError<TwitchMessage>) -> ReconnectHandlerErr {
        ReconnectHandlerErr::Handler(err.to_string())
    }
}

impl Reconnect {
    fn handle(
        &self,
        session: Option<&mut Session>,
        tx: Sender<TwitchMessage>,
    ) -> Result<(), ReconnectHandlerErr> {
        let old_session = match session {
            Some(session) => session,
            None => {
                return Err(ReconnectHandlerErr::Session(
                    "reconnect handler always needs session".to_owned(),
                ))
            }
        };

        let url = self.reconnect_url();
        let mut new_session = crate::get_session(Some(url))
            .map_err(|err| ReconnectHandlerErr::Session(err.to_string()))?;

        loop {
            let msg = new_session.socket().read_message()?;
            let msg_raw = msg.to_text()?.to_owned();
            let msg: TwitchMessage = match serde_json::from_str(&msg_raw) {
                Ok(msg) => msg,
                Err(_) => continue,
            };

            let message_id = msg.id();

            if new_session.handled().contains(&message_id) {
                println!("Duplicate message: {:#?}", msg);
                continue;
            }

            let is_welcome: bool = matches!(msg, TwitchMessage::Welcome(_));

            match msg.handle(Some(&mut new_session), tx.clone()) {
                Ok(_) => {}
                Err(err) => match err {
                    HandlerErr::Welcome(err) => match err {
                        WelcomeHandlerErr::NoKeepalive(_) => {}
                        _ => return Err(HandlerErr::from(err).into()),
                    },
                    _ => return Err(err.into()),
                },
            };

            tx.send(msg)?;

            new_session.handled().push(message_id.to_owned());

            if is_welcome {
                break;
            };
        }

        loop {
            let msg = match old_session.socket().read_message() {
                Ok(msg) => msg,
                Err(_) => break,
            };
            old_session.socket().close(None)?;
            let msg_raw = msg.to_text()?.to_owned();
            let msg: TwitchMessage = match serde_json::from_str(&msg_raw) {
                Ok(msg) => msg,
                Err(_) => continue,
            };

            let message_id = msg.id();

            if old_session.handled().contains(&message_id) {
                println!("Duplicate message: {:#?}", msg);
                continue;
            }

            msg.handle(Some(old_session), tx.clone())?;

            tx.send(msg)?;

            old_session.handled().push(message_id.to_owned());
        }

        *old_session = new_session;
        Ok(())
    }
}
