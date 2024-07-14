use crate::storage::aggtrade_storage::AggTradeStorage;
use crate::websocket::client::subscribe::subscribe_to_streams;
use crate::websocket::client::unsubscribe::unsubscribe_from_streams;
use crate::websocket::handler::aggtrade_handler::handle_aggtrade_messages;
use crate::websocket::ping::start_ping;
use crate::websocket::shutdown::handle_shutdown;
use futures_util::StreamExt;
use std::sync::{Arc, RwLock};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::connect_async;

pub async fn run(
    url: &str,
    streams: &Vec<String>,
    base_id: u64,
    storage: &Arc<RwLock<AggTradeStorage>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
    let (ws_shutdown_tx, ws_shutdown_rx) = oneshot::channel();

    // Spawn a task to handle shutdown
    tokio::spawn(async move {
        handle_shutdown(&shutdown_tx).await;
    });

    // Connect to WebSocket
    let (ws_stream, _) = connect_async(url).await?;
    println!("WebSocket connected");

    let (mut write, mut read) = ws_stream.split();

    // Subscribe to streams
    subscribe_to_streams(&mut write, &streams, base_id).await?;
    let storage_clone = Arc::clone(storage);
    // Handle incoming messages
    let handle = tokio::spawn(async move {
        handle_aggtrade_messages(&mut read, &storage_clone, &mut shutdown_rx).await;
        let _ = ws_shutdown_tx.send(());
    });

    // Start ping
    start_ping(&mut write, ws_shutdown_rx).await;
    // Await the handle to ensure proper handling
    handle.await?;

    // Unsubscribe from streams
    unsubscribe_from_streams(&mut write, &streams, base_id + 1000).await?;

    Ok(())
}
