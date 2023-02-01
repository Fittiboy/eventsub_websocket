#![allow(clippy::uninlined_format_args)]

use std::sync::{mpsc::Sender, Arc, Mutex};
use std::thread;
use std::time::Duration;
pub use tungstenite::{
    connect,
    protocol::frame::{coding::CloseCode, CloseFrame},
};
use url::Url;

use crate::error::*;
use crate::types::{EventResult, Session, Socket, TwitchMessage};

pub use serde_json::from_str as parse_message;

pub mod error;
pub mod handlers;
pub mod types;

pub fn listen_loop(
    eventsub_session: Arc<Mutex<Session>>,
    message_forwarder: &Sender<TwitchMessage>,
    reconnect_to_twitch: bool,
    close_old_connection: bool,
) -> std::result::Result<(), EventSubErr> {
    // The close message needs to be sent if the connection is supposed to be replaced, but after
    // sending the close message, other messages can still come in before Twitch sends a close
    // message in return.
    let mut sent_close_message = false;
    loop {
        if close_old_connection && !sent_close_message {
            eventsub_session.lock()?.socket.close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: "Received reconnect message.".into(),
            }))?;
            sent_close_message = true;
        }

        let msg = {
            let session = &mut eventsub_session.lock()?;
            let reconnect_url = session.url.take();
            let socket = &mut session.socket;
            if !socket.can_read() && close_old_connection {
                break;
            }
            match socket.read_message() {
                Ok(msg) => msg,
                Err(err) => match err {
                    tungstenite::Error::ConnectionClosed => return Err(err.into()),
                    tungstenite::Error::Io(err) => {
                        println!("Connection lost\n\t{}\n\tReconnecting...", err);
                        attempt_reconnection(socket, reconnect_url)?;
                        continue;
                    }
                    _ => {
                        return Err(err.into());
                    }
                },
            }
        };

        let msg_raw = msg.to_text()?.to_owned();
        let msg: TwitchMessage = match serde_json::from_str(&msg_raw) {
            Ok(msg) => msg,
            Err(_) => {
                println!("Received unhandled message type from Twitch, ignoring...");
                continue;
            }
        };

        if eventsub_session.lock()?.handled.contains(&msg.id()) {
            println!("Duplicate message: {:#?}", msg);
            continue;
        }

        let message_is_welcome: bool = matches!(msg, TwitchMessage::Welcome(_));

        if let Err(err) = msg.handle(Some(Arc::clone(&eventsub_session)), message_forwarder) {
            match err {
                HandlerErr::Welcome(WelcomeHandlerErr::NoKeepalive(_)) => {
                    // A Welcome message in response to a reconnection attempt is not supposed
                    // to carry a new keepalive time, but the keepalive time should *not* be
                    // missing for an initial Welcome message.
                    if !reconnect_to_twitch {
                        return Err(err.into());
                    }
                }
                _ => return Err(err.into()),
            }
        };

        eventsub_session.lock()?.handled.push(msg.id());
        message_forwarder.send(msg)?;

        if message_is_welcome && reconnect_to_twitch {
            // If `reconnect_to_twitch` was set, only the `Welcome` message needs special handling.
            // Afterwards, handling will be handed over to the original loop, by replacing the
            // socket of the original session.
            break;
        };
    }
    Ok(())
}

fn attempt_reconnection(
    socket: &mut Socket,
    reconnect_url: Option<String>,
) -> std::result::Result<(), EventSubErr> {
    let mut reconnect_wait_time_seconds = 1;
    loop {
        match get_session(reconnect_url.clone()) {
            Ok(new_session) => {
                let new_socket = &mut new_session.lock()?.socket;
                std::mem::swap(socket, new_socket);
                println!("Reconnected!");
                break Ok(());
            }
            Err(err) => {
                println!(
                    "Failed to connect:\n\t{}\n\tRetrying in {}s...",
                    err, reconnect_wait_time_seconds
                );
                thread::sleep(Duration::from_secs(reconnect_wait_time_seconds));
                reconnect_wait_time_seconds *= 2;
            }
        }
    }
}

pub fn event_handler(
    url: Option<String>,
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

pub fn get_session(url: Option<String>) -> Result<Arc<Mutex<Session>>, EventSubErr> {
    let to_parse;
    if let Some(url) = url {
        to_parse = url;
    } else {
        to_parse = "wss://eventsub-beta.wss.twitch.tv/ws".to_string();
    }
    let (socket, _) = connect(Url::parse(&to_parse)?)?;
    Ok(Arc::new(Mutex::new(Session::new(socket, Some(to_parse)))))
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
            command.arg("--reconnect").arg("1");
        }
        command.spawn().expect("failed to start server")
    }

    #[test]
    fn connect_to_mock() {
        let mut handle = start_server(false, 8080);
        thread::sleep(std::time::Duration::from_secs(1));
        let session = get_session(Some("ws://localhost:8080/eventsub".to_owned())).unwrap();
        session
            .lock()
            .unwrap()
            .socket
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
        let res = event_handler(Some("ws://localhost:8082/eventsub".to_owned()), tx).unwrap();
        loop {
            let msg: TwitchMessage = rx.recv().map_err(|err| format!("{}", err)).unwrap();
            match msg {
                TwitchMessage::Welcome(_) => {
                    res.session
                        .lock()
                        .unwrap()
                        .socket
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
        let session = get_session(Some("ws://localhost:8084/eventsub".to_owned())).unwrap();
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
