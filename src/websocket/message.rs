use futures_util::StreamExt;
use tokio_tungstenite::tungstenite::protocol::Message;

pub async fn handle_messages(
    mut read: impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
) {
    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(text)) = msg {
            println!("Received: {}", text);
        }
    }
}
