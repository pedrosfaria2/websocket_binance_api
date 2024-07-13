use crate::websocket::message::handle_messages;
use crate::websocket::ping::start_ping;
use crate::websocket::shutdown::handle_shutdown;
use crate::websocket::subscriptions::{subscribe_message, unsubscribe_message};
use crate::storage::aggtrade_storage::AggTradeStorage;
use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub const BINANCE_WS_URL: &str = "wss://stream.binance.com:9443/ws/";
pub const BINANCE_WS_COMBINED_URL: &str = "wss://stream.binance.com:9443/stream?streams=";

/// Runs the WebSocket client with the specified URL and streams.
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

/// Subscribes to the specified streams.
pub async fn subscribe_to_streams<S>(
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
pub async fn unsubscribe_from_streams<S>(
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


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::sync::mpsc;
    use tokio_tungstenite::tungstenite::protocol::Message;

    struct MockSink {
        tx: mpsc::UnboundedSender<Message>,
    }

    impl MockSink {
        fn new() -> (Self, mpsc::UnboundedReceiver<Message>) {
            let (tx, rx) = mpsc::unbounded_channel();
            (MockSink { tx }, rx)
        }
    }

    impl futures_util::Sink<Message> for MockSink {
        type Error = mpsc::error::SendError<Message>;

        fn poll_ready(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }

        fn start_send(self: std::pin::Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
            self.tx.send(item).map_err(|e| e.into())
        }

        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }

        fn poll_close(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn test_subscribe_to_streams() {
        let (mut mock_sink, mut rx) = MockSink::new();
        let streams = vec!["btcusdt@aggTrade".to_string(), "ethusdt@trade".to_string()];
        let base_id = 1;

        subscribe_to_streams(&mut mock_sink, &streams, base_id)
            .await
            .unwrap();

        let msg = rx.recv().await.unwrap();
        if let Message::Text(text) = msg {
            let message_json: serde_json::Value = serde_json::from_str(&text).unwrap();
            let expected_message = json!({
                "method": "SUBSCRIBE",
                "params": streams,
                "id": base_id
            });
            assert_eq!(message_json, expected_message);
        } else {
            panic!("Expected text message");
        }
    }


    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::json;
        use tokio::sync::mpsc;
        use tokio_tungstenite::tungstenite::protocol::Message;

        struct MockSink {
            tx: mpsc::UnboundedSender<Message>,
        }

        impl MockSink {
            fn new() -> (Self, mpsc::UnboundedReceiver<Message>) {
                let (tx, rx) = mpsc::unbounded_channel();
                (MockSink { tx }, rx)
            }
        }

        impl futures_util::Sink<Message> for MockSink {
            type Error = mpsc::error::SendError<Message>;

            fn poll_ready(
                self: std::pin::Pin<&mut Self>,
                _: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Result<(), Self::Error>> {
                std::task::Poll::Ready(Ok(()))
            }

            fn start_send(self: std::pin::Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
                self.tx.send(item).map_err(|e| e.into())
            }

            fn poll_flush(
                self: std::pin::Pin<&mut Self>,
                _: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Result<(), Self::Error>> {
                std::task::Poll::Ready(Ok(()))
            }

            fn poll_close(
                self: std::pin::Pin<&mut Self>,
                _: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Result<(), Self::Error>> {
                std::task::Poll::Ready(Ok(()))
            }
        }

        #[tokio::test]
        async fn test_subscribe_to_streams() {
            let (mut mock_sink, mut rx) = MockSink::new();
            let streams = vec!["btcusdt@aggTrade".to_string(), "ethusdt@trade".to_string()];
            let base_id = 1;

            subscribe_to_streams(&mut mock_sink, &streams, base_id)
                .await
                .unwrap();

            let msg = rx.recv().await.unwrap();
            if let Message::Text(text) = msg {
                let message_json: serde_json::Value = serde_json::from_str(&text).unwrap();
                let expected_message = json!({
                "method": "SUBSCRIBE",
                "params": streams,
                "id": base_id
            });
                assert_eq!(message_json, expected_message);
            } else {
                panic!("Expected text message");
            }
        }

        #[tokio::test]
        async fn test_unsubscribe_from_streams() {
            let (mut mock_sink, mut rx) = MockSink::new();
            let streams = vec!["btcusdt@aggTrade".to_string(), "ethusdt@trade".to_string()];
            let base_id = 1001;

            unsubscribe_from_streams(&mut mock_sink, &streams, base_id)
                .await
                .unwrap();

            let msg = rx.recv().await.unwrap();
            if let Message::Text(text) = msg {
                let message_json: serde_json::Value = serde_json::from_str(&text).unwrap();
                let expected_message = json!({
                "method": "UNSUBSCRIBE",
                "params": streams,
                "id": base_id
            });
                assert_eq!(message_json, expected_message);
            } else {
                panic!("Expected text message");
            }
        }
    }
}
