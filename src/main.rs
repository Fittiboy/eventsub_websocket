use eventsub_websocket::{event_handler, get_session};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = get_session();
    event_handler(&mut session)?;
    Ok(())
}
