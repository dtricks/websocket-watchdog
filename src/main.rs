use futures_util::{SinkExt, StreamExt};
use log::*;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::{net::SocketAddr, time::Duration};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Error};
use tungstenite::{Message, Result};

async fn accept_connection(peer: SocketAddr, stream: TcpStream) {
    if let Err(e) = handle_connection(peer, stream).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    info!("New WebSocket connection: {}", peer);
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut interval = tokio::time::interval(Duration::from_millis(1000));

    let (tx, rx) = crossbeam_channel::unbounded();
    let mut watcher: RecommendedWatcher =
        Watcher::new_immediate(move |res| tx.send(res).unwrap()).unwrap();
    info!("created watcher");
    watcher
        .watch(std::path::Path::new("Readme.md"), RecursiveMode::Recursive)
        .unwrap();
    info!("started watching");
    for res in rx {
        match res {
            Ok(event) => {
                ws_sender
                    .send(Message::Text(format!("{:?}\n", event)))
                    .await?;
                info!("{:?}", event);
            }
            Err(e) => error!("Watch error: {:?}", e),
        }
    }
    //loop {
    //    tokio::select! {
    //        msg = ws_receiver.next() => {
    //            match msg {
    //                Some(msg) => {
    //                    let msg = msg?;
    //                    if msg.is_text() ||msg.is_binary() {
    //                        ws_sender.send(msg).await?;
    //                    } else if msg.is_close() {
    //                        break;
    //                    }
    //                }
    //                None => break,
    //            }
    //        }
    //        _ = interval.tick() => {
    //            ws_sender.send(Message::Text("tick\n".to_owned())).await?;
    //            info!("Tick happened");
    //        }
    //    }
    //}

    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let addr = "127.0.0.1:9002";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);

        tokio::spawn(accept_connection(peer, stream));
    }
}
