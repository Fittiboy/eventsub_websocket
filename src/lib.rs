#![allow(clippy::uninlined_format_args)]

use std::sync::{mpsc::Sender, Arc, Mutex};
use std::thread;
pub use tungstenite::{
    connect,
    protocol::frame::{coding::CloseCode, CloseFrame},
};
use url::Url;

use crate::handlers::error::*;
use crate::types::{EventResult, Session, TwitchMessage};

pub use crate::handlers::error;
pub use serde_json::from_str as parse_message;

pub mod handlers;
pub mod types;

pub fn listen_loop<'a>(
    session: &'a mut Session,
    tx: &Sender<TwitchMessage>,
    reconnect: bool,
    close_old: bool,
) -> std::result::Result<(), EventSubErr> {
    loop {
        let msg = session.socket.lock()?.read_message()?;
        let msg_raw = msg.to_text()?.to_owned();
        let msg: TwitchMessage = match serde_json::from_str(&msg_raw) {
            Ok(msg) => msg,
            Err(_) => continue,
        };

        if close_old {
            session.socket.lock()?.close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: "Received reconnect message.".into(),
            }))?;
        }

        let message_id = msg.id();

        if session.handled.contains(&message_id) {
            println!("Duplicate message: {:#?}", msg);
            continue;
        }

        let is_welcome: bool = matches!(msg, TwitchMessage::Welcome(_));

        match msg.handle(Some(session), tx) {
            Ok(_) => {}
            Err(err) => match err {
                HandlerErr::Welcome(err) => match err {
                    WelcomeHandlerErr::NoKeepalive(_) => {
                        // A Welcome message in response to a reconnection attempt is not supposed
                        // to carry a new keepalive time, but the keepalive time should *not* be
                        // missing for an initial Welcome message.
                        if !reconnect {
                            return Err(HandlerErr::from(err).into());
                        }
                    }
                    _ => return Err(HandlerErr::from(err).into()),
                },
                _ => return Err(err.into()),
            },
        };

        tx.send(msg)?;

        session.handled.push(message_id.to_owned());

        if is_welcome && reconnect {
            break;
        };
    }
    Ok(())
}

pub fn event_handler(
    url: Option<&str>,
    tx: Sender<TwitchMessage>,
) -> std::result::Result<EventResult, EventSubErr> {
    let mut session = get_session(url)?;
    let socket = session.socket.clone();
    let listener =
        thread::Builder::new()
            .name("listener".into())
            .spawn(move || -> Result<(), String> {
                listen_loop(&mut session, &tx, false, false)?;
                Ok(())
            })?;
    Ok(EventResult { listener, socket })
}

pub fn get_session(url: Option<&str>) -> Result<Session, EventSubErr> {
    let to_parse;
    if let Some(url) = url {
        to_parse = url;
    } else {
        to_parse = "wss://eventsub-beta.wss.twitch.tv/ws";
    }
    let (socket, _) = connect(Url::parse(to_parse)?)?;
    let socket = Arc::new(Mutex::new(socket));
    Ok(Session::new(socket))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::{self, Receiver, Sender};

    #[test]
    fn connect_to_mock() {
        let session = get_session(Some("ws://localhost:8080/eventsub")).unwrap();
        session
            .socket
            .lock()
            .unwrap()
            .close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: "Closing after connect test.".into(),
            }))
            .unwrap();
    }

    #[test]
    fn handle_welcome_message() {
        let (tx, rx): (Sender<TwitchMessage>, Receiver<TwitchMessage>) = mpsc::channel();
        let res = event_handler(Some("ws://localhost:8080/eventsub"), tx).unwrap();
        loop {
            let msg: TwitchMessage = rx.recv().map_err(|err| format!("{}", err)).unwrap();
            match msg {
                TwitchMessage::Welcome(_) => {
                    res.socket
                        .lock()
                        .unwrap()
                        .close(Some(CloseFrame {
                            code: CloseCode::Normal,
                            reason: "Closing after Welcome test.".into(),
                        }))
                        .unwrap();
                    return ();
                }
                _ => {}
            }
        }
    }

    #[test]
    fn handle_reconnect_message() {
        let mut welcome_count = 0;
        let (tx, rx): (Sender<TwitchMessage>, Receiver<TwitchMessage>) = mpsc::channel();
        let res = event_handler(Some("ws://localhost:8080/eventsub"), tx).unwrap();
        loop {
            let msg: TwitchMessage = rx.recv().map_err(|err| format!("{}", err)).unwrap();
            match msg {
                TwitchMessage::Welcome(_) => {
                    welcome_count += 1;
                    if welcome_count > 1 {
                        res.socket
                            .lock()
                            .unwrap()
                            .close(Some(CloseFrame {
                                code: CloseCode::Normal,
                                reason: "Closing after reconnect test.".into(),
                            }))
                            .unwrap();
                        return ();
                    }
                }
                _ => {}
            }
        }
    }
}
