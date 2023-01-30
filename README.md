# EventSub WebSocket
This library serves as a client for [Twitch EventSub](https://dev.twitch.tv/docs/eventsub/) subscriptions via WebSocket.  
It is used as the backend for my personal project, the [Pond opener 3000™](https://github.com/Fittiboy/rust_fishinge).  
It supports reconnects both in the case of Twitch sending a [`Reconnect`](https://dev.twitch.tv/docs/eventsub/handling-websocket-events/#reconnect-message) message, as well as underlying connection issues.

## NOTE
This is a personal project for me, to experiment with Rust. If you want to interact with Twitch services, use the [twitch_api crate](https://crates.io/crates/twitch_api)!
