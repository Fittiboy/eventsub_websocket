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

pub fn listen_loop(
    session: Arc<Mutex<Session>>,
    tx: &Sender<TwitchMessage>,
    reconnect: bool,
    close_old: bool,
) -> std::result::Result<(), EventSubErr> {
    let mut closed = false;
    loop {
        if close_old && !closed {
            session.lock()?.socket.lock()?.close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: "Received reconnect message.".into(),
            }))?;
            closed = true;
        }

        let msg = {
            let session = session.lock()?;
            let mut socket = session.socket.lock()?;
            if !socket.can_read() {
                break;
            }
            socket.read_message()?
        };
        let msg_raw = msg.to_text()?.to_owned();
        let msg: TwitchMessage = match serde_json::from_str(&msg_raw) {
            Ok(msg) => msg,
            Err(_) => continue,
        };

        let message_id = msg.id();

        if session.lock()?.handled.contains(&message_id) {
            println!("Duplicate message: {:#?}", msg);
            continue;
        }

        let is_welcome: bool = matches!(msg, TwitchMessage::Welcome(_));

        match msg.handle(Some(Arc::clone(&session)), tx) {
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

        session.lock()?.handled.push(message_id.to_owned());

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
    let session = get_session(url)?;
    let move_sess = Arc::clone(&session);
    let listener =
        thread::Builder::new()
            .name("listener".into())
            .spawn(move || -> Result<(), String> {
                listen_loop(move_sess, &tx, false, false)?;
                Ok(())
            })?;
    Ok(EventResult { listener, session })
}

pub fn get_session(url: Option<&str>) -> Result<Arc<Mutex<Session>>, EventSubErr> {
    let to_parse;
    if let Some(url) = url {
        to_parse = url;
    } else {
        to_parse = "wss://eventsub-beta.wss.twitch.tv/ws";
    }
    let (socket, _) = connect(Url::parse(to_parse)?)?;
    let socket = Arc::new(Mutex::new(socket));
    Ok(Arc::new(Mutex::new(Session::new(socket))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Child, Command};
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::thread;

    static COMMAND: &str = "./scripts/test_server.sh";

    fn start_server(reconnect: bool, port: u32) -> Child {
        let mut command = Command::new(COMMAND);
        command.arg("--port").arg(&format!("{}", port));
        if reconnect {
            command.arg("--reconnect").arg("3");
        }
        command.spawn().expect("failed to start server")
    }

    #[test]
    fn connect_to_mock() {
        let mut handle = start_server(false, 8080);
        thread::sleep(std::time::Duration::from_secs(1));
        let session = get_session(Some("ws://localhost:8080/eventsub")).unwrap();
        session
            .lock()
            .unwrap()
            .socket
            .lock()
            .unwrap()
            .close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: "Closing after connect test.".into(),
            }))
            .unwrap();
        handle.kill().unwrap();
    }

    #[test]
    fn handle_welcome_message() {
        let mut handle = start_server(false, 8082);
        thread::sleep(std::time::Duration::from_secs(1));
        let (tx, rx): (Sender<TwitchMessage>, Receiver<TwitchMessage>) = mpsc::channel();
        let res = event_handler(Some("ws://localhost:8082/eventsub"), tx).unwrap();
        loop {
            let msg: TwitchMessage = rx.recv().map_err(|err| format!("{}", err)).unwrap();
            match msg {
                TwitchMessage::Welcome(_) => {
                    res.session
                        .lock()
                        .unwrap()
                        .socket
                        .lock()
                        .unwrap()
                        .close(Some(CloseFrame {
                            code: CloseCode::Normal,
                            reason: "Closing after Welcome test.".into(),
                        }))
                        .unwrap();
                    break;
                }
                _ => {}
            }
        }
        handle.kill().unwrap();
    }

    #[test]
    fn handle_reconnect_message() {
        let mut handle = start_server(true, 8084);
        thread::sleep(std::time::Duration::from_secs(1));
        let mut welcome_count = 0;
        let (tx, rx): (Sender<TwitchMessage>, Receiver<TwitchMessage>) = mpsc::channel();
        let session = get_session(Some("ws://localhost:8084/eventsub")).unwrap();
        let tx_clone = tx.clone();
        let move_sess = Arc::clone(&session);
        thread::Builder::new()
            .name("listener".into())
            .spawn(move || -> Result<(), String> {
                listen_loop(move_sess, &tx_clone, false, false)?;
                Ok(())
            })
            .unwrap();
        loop {
            let msg: TwitchMessage = rx.recv().map_err(|err| format!("{}", err)).unwrap();
            match msg {
                TwitchMessage::Welcome(_) => {
                    welcome_count += 1;
                }
                TwitchMessage::Keepalive(_) => {
                    if welcome_count >= 2 {
                        // Verify that the new connection is still healthy
                        session
                            .lock()
                            .unwrap()
                            .socket
                            .lock()
                            .unwrap()
                            .close(Some(CloseFrame {
                                code: CloseCode::Normal,
                                reason: "Closing after reconnect test.".into(),
                            }))
                            .unwrap();
                        break;
                    }
                }
                _ => {}
            }
        }
        handle.kill().unwrap();
    }
}
