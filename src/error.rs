use crate::types::Session;
use crate::TwitchMessage;
use std::io;
use std::sync::mpsc::SendError;
use std::sync::{MutexGuard, PoisonError};
use thiserror::Error;
use url::ParseError;

pub mod casting;

/// Top level error type returned by the API
#[derive(Error, Debug)]
pub enum EventSubErr {
    #[error("general handler error: {0}")]
    GeneralHandler(HandlerErr),
    #[error("socket error: {0}")]
    Socket(tungstenite::Error),
    #[error("session error: {0}")]
    Session(SessionErr),
    #[error("error sending through channel: {0}")]
    Sending(SendError<TwitchMessage>),
    #[error("error creating listener thread: {0}")]
    Thread(io::Error),
    #[error("session mutex has been poisoned: {0}")]
    Poison(String),
}

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
    CannotSetKeepalive(KeepaliveErr),
    #[error("session mutex has been poisoned: {0}")]
    Poison(String),
}

#[derive(Error, Debug)]
pub enum ReconnectHandlerErr {
    #[error("session error while reconnecting: {0}")]
    Session(String),
    #[error("general error while reconnecting: {0}")]
    Handler(String),
    #[error("connection error while reconnecting: {0}")]
    Connection(tungstenite::Error),
    #[error("session mutex has been poisoned: {0}")]
    Poison(String),
}

#[derive(Error, Debug)]
pub enum SessionErr {
    #[error("error parsing url: {0}")]
    Parse(ParseError),
    #[error("connection error: {0}")]
    Connect(tungstenite::Error),
}

#[derive(Error, Debug)]
pub enum KeepaliveErr {
    #[error("error setting the socket's timeout to keepalive: {0}")]
    Timeout(io::Error),
    #[error("session mutex has been poisoned: {0}")]
    Poison(String),
}
