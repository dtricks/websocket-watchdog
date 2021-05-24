# Websocket Watchdog

This tool watches the given server filesystem path (recursively) and communicates filesystem events (file modified, accessed, metadata changed) via a websocket API to all clients which previously connected to the server.

## Run server

```bash
RUST_LOG=info cargo run .
```

## Run CLI client

```bash
cargo run --example client ws://127.0.0.1:9002
```

## Webbrowser JavaScript-client

```bash
firefox examples/client-js/index.html
```

## TODO

- [x] fix: events are not distributed to all clients
- [x] fix: spin loop in receiver
