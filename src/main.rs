use futures_util::{SinkExt, StreamExt};
use log::*;
use multiqueue::wait::BlockingWait;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::env;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, accept_async_with_config, tungstenite::Error};
use tungstenite::{protocol::WebSocketConfig, Message, Result};

async fn accept_connection(
    peer: SocketAddr,
    stream: TcpStream,
    rx: multiqueue::BroadcastReceiver<notify::event::Event>,
) {
    if let Err(e) = handle_connection(peer, stream, rx).await {
        match e {
            Error::Protocol(_) | Error::Utf8 => (),
            Error::ConnectionClosed => {
                info!("Connection closed with {}", peer);
            }
            err => error!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    rx: multiqueue::BroadcastReceiver<notify::event::Event>,
) -> Result<()> {
    let wsc = WebSocketConfig {
        max_send_queue: None,
        max_message_size: None,
        max_frame_size: None,
        accept_unmasked_frames: true,
    };
    let ws_stream = accept_async_with_config(stream, Some(wsc))
        .await
        .expect("Failed to accept");
    info!("New WebSocket connection: {}", peer);
    let (mut ws_sender, mut _ws_receiver) = ws_stream.split();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

    loop {
        let res = rx.try_recv();
        match res {
            Ok(event) => {
                ws_sender
                    .send(Message::Text(format!("{:?}\n", event)))
                    .await?
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                interval.tick().await;
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                break;
            }
        }
    }
    ws_sender.close().await?;
    info!("Closed websocket connection to {}", peer);
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let path_str = env::args().nth(1).expect("First argument has to be a path");

    //let addr = "127.0.0.1:9002";
    let addr = "0.0.0.0:9002";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);
    let (tx, rx) = multiqueue::broadcast_queue_with(1000, BlockingWait::new());
    let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| match res {
        Ok(event) => {
            info!("{:?}", event);
            use std::sync::mpsc::TrySendError::*;
            match tx.try_send(event) {
                Ok(_) => (),
                Err(Full(e)) => error!("Multiqueue full error {:?}", e),
                Err(Disconnected(e)) => error!("Multiqueue disconnected error {:?}", e),
            }
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
