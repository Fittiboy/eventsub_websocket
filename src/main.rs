use eventsub_websocket::{event_handler, get_session};

fn main() {
    let mut session = get_session();
    event_handler(&mut session);
}
