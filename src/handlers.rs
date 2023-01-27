use crate::handlers::error::*;
use crate::listen_loop;
use crate::types::{Reconnect, Session, TwitchMessage, Welcome};
use std::sync::mpsc::Sender;

pub mod error;

impl TwitchMessage {
    pub fn handle(
        &self,
        session: Option<&mut Session>,
        tx: &Sender<TwitchMessage>,
    ) -> Result<(), HandlerErr> {
        match self {
            TwitchMessage::Welcome(msg) => Ok(msg.handle(session)?),
            TwitchMessage::Reconnect(msg) => Ok(msg.handle(session, tx)?),
            _ => Ok(()),
        }
    }
}

impl Welcome {
    fn handle(&self, session: Option<&mut Session>) -> Result<(), WelcomeHandlerErr> {
        if let Some(session) = session {
            session.id = self.payload.session.id.to_string();
            let keepalive = self.payload.session.keepalive_timeout_seconds.as_u64();
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

impl Reconnect {
    fn handle(
        &self,
        session: Option<&mut Session>,
        tx: &Sender<TwitchMessage>,
    ) -> Result<(), ReconnectHandlerErr> {
        let old_session = match session {
            Some(session) => session,
            None => {
                return Err(ReconnectHandlerErr::Session(
                    "reconnect handler always needs session".to_owned(),
                ))
            }
        };

        let url = &self.payload.session.reconnect_url;
        let mut new_session = crate::get_session(Some(url))
            .map_err(|err| ReconnectHandlerErr::Session(err.to_string()))?;

        listen_loop(&mut new_session, tx, true, false)
            .map_err(|err| ReconnectHandlerErr::Handler(err.to_string()))?;

        listen_loop(old_session, tx, false, true)
            .map_err(|err| ReconnectHandlerErr::Handler(err.to_string()))?;

        *old_session = new_session;
        Ok(())
    }
}
