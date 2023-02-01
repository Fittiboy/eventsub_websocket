use crate::error::*;

// Implementations for the `EventSubErr` Error type
impl From<EventSubErr> for String {
    fn from(err: EventSubErr) -> String {
        err.to_string()
    }
}

impl From<PoisonError<MutexGuard<'_, Session>>> for EventSubErr {
    fn from(err: PoisonError<MutexGuard<'_, Session>>) -> Self {
        EventSubErr::Poison(err.to_string())
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

impl From<ParseError> for EventSubErr {
    fn from(err: ParseError) -> Self {
        EventSubErr::Session(SessionErr::Parse(err))
    }
}

// Implementations for the `HandlerErr` Error type
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

// Implementations for the `WelcomeHandlerErr` Error type
impl From<KeepaliveErr> for WelcomeHandlerErr {
    fn from(err: KeepaliveErr) -> Self {
        WelcomeHandlerErr::CannotSetKeepalive(err)
    }
}

impl From<PoisonError<MutexGuard<'_, Session>>> for WelcomeHandlerErr {
    fn from(err: PoisonError<MutexGuard<'_, Session>>) -> Self {
        WelcomeHandlerErr::Poison(err.to_string())
    }
}

// Implementations for the `ReconnectHandlerErr` Error type
impl From<tungstenite::Error> for ReconnectHandlerErr {
    fn from(err: tungstenite::Error) -> ReconnectHandlerErr {
        ReconnectHandlerErr::Connection(err)
    }
}

impl From<PoisonError<MutexGuard<'_, Session>>> for ReconnectHandlerErr {
    fn from(err: PoisonError<MutexGuard<'_, Session>>) -> Self {
        ReconnectHandlerErr::Poison(err.to_string())
    }
}

impl From<HandlerErr> for ReconnectHandlerErr {
    fn from(err: HandlerErr) -> ReconnectHandlerErr {
        ReconnectHandlerErr::Handler(Box::new(err))
    }
}

impl From<EventSubErr> for ReconnectHandlerErr {
    fn from(err: EventSubErr) -> ReconnectHandlerErr {
        ReconnectHandlerErr::EventSub(Box::new(err))
    }
}

impl From<SendError<TwitchMessage>> for ReconnectHandlerErr {
    fn from(err: SendError<TwitchMessage>) -> ReconnectHandlerErr {
        ReconnectHandlerErr::Sending(err)
    }
}

impl From<url::ParseError> for ReconnectHandlerErr {
    fn from(err: url::ParseError) -> ReconnectHandlerErr {
        ReconnectHandlerErr::Url(err)
    }
}

// Implementations for the `SessionErr` Error type
impl From<tungstenite::Error> for SessionErr {
    fn from(err: tungstenite::Error) -> Self {
        SessionErr::Connect(err)
    }
}

// Implementations for the `KeepaliveErr` Error type
impl From<io::Error> for KeepaliveErr {
    fn from(err: io::Error) -> Self {
        KeepaliveErr::Timeout(err)
    }
}
