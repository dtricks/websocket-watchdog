use futures_util::{SinkExt, StreamExt};
use log::*;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::env;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Error};
use tungstenite::{Message, Result};

async fn accept_connection(
    peer: SocketAddr,
    stream: TcpStream,
    rx: multiqueue::BroadcastReceiver<notify::event::Event>,
) {
    if let Err(e) = handle_connection(peer, stream, rx).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    rx: multiqueue::BroadcastReceiver<notify::event::Event>,
) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    info!("New WebSocket connection: {}", peer);
    let (mut ws_sender, mut _ws_receiver) = ws_stream.split();

    for event in rx {
        ws_sender
            .send(Message::Text(format!("{:?}\n", event)))
            .await?;
        info!("{:?}", event);
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let path_str = env::args().nth(1).expect("First argument has to be a path");

    let addr = "127.0.0.1:9002";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);
    //let (tx, rx) = crossbeam_channel::unbounded();
    let (tx, rx) = multiqueue::broadcast_queue(200);
    let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| match res {
        Ok(event) => {
            tx.try_send(event).unwrap();
        }
        Err(e) => {
            error!("Watch Error {}", e);
        }
    })
    .unwrap();
    info!("created watcher");
    watcher
        .watch(std::path::Path::new(&path_str), RecursiveMode::Recursive)
        .unwrap();
    info!("started watching");

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);

        tokio::spawn(accept_connection(peer, stream, rx.add_stream()));
    }
}
