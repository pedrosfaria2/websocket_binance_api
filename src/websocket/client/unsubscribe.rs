use crate::websocket::subscriptions::unsubscribe_message;
use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::protocol::Message;

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
