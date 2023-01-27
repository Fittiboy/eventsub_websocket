#!/usr/bin/env bash
if ! [ -x "$(command -v twitch)" ]; then
    echo >&2 "Error: twitch-cli not installed"
    exit 1
fi

twitch event start-websocket-server --reconnect 3
