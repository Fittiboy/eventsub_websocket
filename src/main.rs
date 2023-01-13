use eventsub_websocket::{event_handler, get_session, handle_welcome};

fn main() {
    let mut socket = get_session();
    let id = handle_welcome(&mut socket);
    event_handler(&mut socket);
}
