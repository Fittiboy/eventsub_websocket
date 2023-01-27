use crate::TwitchMessage;
use std::io;
use std::sync::mpsc::SendError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HandlerErr {
    #[error("error handling welcome message: {0}")]
    Welcome(WelcomeHandlerErr),
    #[error("error handling erconnect message: {0}")]
    Reconnect(ReconnectHandlerErr),
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
