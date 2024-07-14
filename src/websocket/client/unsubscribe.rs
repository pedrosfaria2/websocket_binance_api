use crate::websocket::subscriptions::unsubscribe_message;
use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::protocol::Message;

/// Unsubscribe from specified streams.
pub async fn unsubscribe_from_streams<S>(
    write: &mut S,
    streams: &Vec<String>,
    base_id: u64,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: SinkExt<Message> + Unpin,
    <S as futures_util::Sink<Message>>::Error: std::error::Error + 'static,
{
    // Create the unsubscription message.
    let unsubscribe_msg = unsubscribe_message(streams, base_id);
    println!("Unsubscribe Payload: {}", unsubscribe_msg);

    // Send the unsubscription message.
    write.send(Message::Text(unsubscribe_msg)).await?;
    Ok(())
}
