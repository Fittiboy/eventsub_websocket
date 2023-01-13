use crate::types::*;

pub fn handle_welcome(msg: Welcome, session: &mut Session) {
    session.set_id(msg.payload.session.id.to_owned());
    println!("Welcome received: {:#?}", msg);
}

pub fn handle_keepalive(msg: Keepalive) {
    println!("Keepalive received: {:#?}", msg);
}

pub fn handle_notification(msg: Notification) {
    println!("Notifiaction received: {:#?}", msg);
}

pub fn handle_reconnect(msg: Reconnect) {
    println!("Notifiaction received: {:#?}", msg);
}

pub fn handle_revocation(msg: Revocation) {
    println!("Notifiaction received: {:#?}", msg);
}
