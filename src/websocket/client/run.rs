use crate::websocket::handler::handle_messages;
use crate::websocket::ping::start_ping;
use crate::websocket::shutdown::handle_shutdown;
use crate::websocket::client::subscribe::subscribe_to_streams;
use crate::websocket::client::unsubscribe::unsubscribe_from_streams;
use crate::storage::aggtrade_storage::AggTradeStorage;
use futures_util::StreamExt;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use tokio_tungstenite::connect_async;

pub async fn run(
    url: &str,
    streams: Vec<String>,
    base_id: u64,
    storage: Arc<Mutex<AggTradeStorage>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let (ws_shutdown_tx, ws_shutdown_rx) = oneshot::channel();

    // Clone storage to move into async block
    let storage_clone = Arc::clone(&storage);

    // Spawn a task to handle shutdown
    tokio::spawn(async move {
        handle_shutdown(shutdown_tx).await;
    });

    // Connect to WebSocket
    let (ws_stream, _) = connect_async(url).await?;
    println!("WebSocket connected");

    let (mut write, read) = ws_stream.split();

    // Subscribe to streams
    subscribe_to_streams(&mut write, &streams, base_id).await?;

    // Handle incoming messages
    let handle = tokio::spawn(async move {
        handle_messages(read, storage_clone).await;
        let _ = ws_shutdown_tx.send(());
    });

    // Start ping
    start_ping(&mut write, shutdown_rx).await;

    // Unsubscribe from streams
    unsubscribe_from_streams(&mut write, &streams, base_id + 1000).await?;

    // Await the handle to ensure proper handling
    ws_shutdown_rx.await.unwrap();
    handle.await?;

    Ok(())
}