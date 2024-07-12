use futures_util::SinkExt;
use std::fmt::Debug;
use tokio::sync::oneshot;
use tokio::time::{interval, Duration};
use tokio_tungstenite::tungstenite::protocol::Message;

pub async fn start_ping<W>(write: &mut W, mut shutdown_rx: oneshot::Receiver<()>)
where
    W: SinkExt<Message> + Unpin,
    <W as futures_util::Sink<Message>>::Error: Debug,
{
    let mut ping_interval = interval(Duration::from_secs(180));
    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
                write.send(Message::Ping(Vec::new())).await.unwrap();
            },
            _ = &mut shutdown_rx => {
                println!("Shutting down WebSocket...");
                write.send(Message::Close(None)).await.unwrap();
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tokio::time::timeout;
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
    async fn test_start_ping() {
        let (mut mock_sink, mut rx) = MockSink::new();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        tokio::spawn(async move {
            start_ping(&mut mock_sink, shutdown_rx).await;
        });

        tokio::time::sleep(Duration::from_secs(1)).await;
        shutdown_tx.send(()).unwrap();

        let mut messages = Vec::new();
        while let Ok(msg) = timeout(Duration::from_secs(1), rx.recv()).await {
            if let Some(msg) = msg {
                messages.push(msg);
            } else {
                break;
            }
        }

        assert!(messages.iter().any(|msg| matches!(msg, Message::Ping(_))));
        assert!(messages.iter().any(|msg| matches!(msg, Message::Close(_))));
    }
}
