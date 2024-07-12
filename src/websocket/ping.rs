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
