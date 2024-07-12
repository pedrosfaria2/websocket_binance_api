use crate::websocket::message::handle_messages;
use crate::websocket::ping::start_ping;
use crate::websocket::shutdown::handle_shutdown;
use crate::websocket::subscriptions::{subscribe_message, unsubscribe_message};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::oneshot;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub const BINANCE_WS_URL: &str = "wss://stream.binance.com:9443/ws/";
pub const BINANCE_WS_COMBINED_URL: &str = "wss://stream.binance.com:9443/stream?streams=";

/// Runs the WebSocket client with the specified URL and streams.
pub async fn run(
    url: &str,
    streams: Vec<String>,
    base_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let (ws_shutdown_tx, ws_shutdown_rx) = oneshot::channel();

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
        handle_messages(read).await;
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

/// Subscribes to the specified streams.
async fn subscribe_to_streams<S>(
    write: &mut S,
    streams: &[String],
    base_id: u64,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: SinkExt<Message> + Unpin,
    <S as futures_util::Sink<Message>>::Error: std::error::Error + 'static,
{
    let subscribe_msg = subscribe_message(streams.to_vec(), base_id);
    println!("Subscribe Payload: {}", subscribe_msg);
    write.send(Message::Text(subscribe_msg)).await?;
    Ok(())
}

/// Unsubscribes from the specified streams.
async fn unsubscribe_from_streams<S>(
    write: &mut S,
    streams: &[String],
    base_id: u64,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: SinkExt<Message> + Unpin,
    <S as futures_util::Sink<Message>>::Error: std::error::Error + 'static,
{
    let unsubscribe_msg = unsubscribe_message(streams.to_vec(), base_id);
    println!("Unsubscribe Payload: {}", unsubscribe_msg);
    write.send(Message::Text(unsubscribe_msg)).await?;
    Ok(())
}
