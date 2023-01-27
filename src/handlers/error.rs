use crate::TwitchMessage;
use std::io;
use std::sync::mpsc::SendError;
use thiserror::Error;
use url::ParseError;

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

#[derive(Error, Debug)]
pub enum SessionErr {
    #[error("error parsing url: {0}")]
    Parse(ParseError),
    #[error("connection error: {0}")]
    Connect(tungstenite::Error),
}

impl From<ParseError> for EventSubErr {
    fn from(err: ParseError) -> Self {
        EventSubErr::Session(SessionErr::Parse(err))
    }
}

impl From<tungstenite::Error> for SessionErr {
    fn from(err: tungstenite::Error) -> Self {
        SessionErr::Connect(err)
    }
}

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
}

impl From<EventSubErr> for String {
    fn from(err: EventSubErr) -> String {
        err.to_string()
    }
}

impl From<HandlerErr> for EventSubErr {
    fn from(err: HandlerErr) -> Self {
        EventSubErr::GeneralHandler(err)
    }
}

impl From<SessionErr> for EventSubErr {
    fn from(err: SessionErr) -> Self {
        EventSubErr::Session(err)
    }
}

impl From<tungstenite::Error> for EventSubErr {
    fn from(err: tungstenite::Error) -> Self {
        EventSubErr::Socket(err)
    }
}

impl From<SendError<TwitchMessage>> for EventSubErr {
    fn from(err: SendError<TwitchMessage>) -> Self {
        EventSubErr::Sending(err)
    }
}

impl From<io::Error> for EventSubErr {
    fn from(err: io::Error) -> Self {
        EventSubErr::Thread(err)
    }
}
