pub fn subscribe_message(streams: Vec<String>, id: u64) -> String {
    // Create subscribe message as a JSON string
    serde_json::json!({
        "method": "SUBSCRIBE",
        "params": streams,
        "id": id
    })
    .to_string()
}

pub fn unsubscribe_message(streams: Vec<String>, id: u64) -> String {
    // Create unsubscribe message as a JSON string
    serde_json::json!({
        "method": "UNSUBSCRIBE",
        "params": streams,
        "id": id
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_subscribe_message() {
        // Test for subscribe message creation
        let streams = vec!["btcusdt@aggTrade".to_string(), "ethusdt@trade".to_string()];
        let id = 1;
        let message = subscribe_message(streams.clone(), id);
        let expected_message = json!({
            "method": "SUBSCRIBE",
            "params": streams,
            "id": id
        });
        let message_json: serde_json::Value = serde_json::from_str(&message).unwrap();
        assert_eq!(message_json, expected_message);
    }

    #[test]
    fn test_unsubscribe_message() {
        // Test for unsubscribe message creation
        let streams = vec!["btcusdt@aggTrade".to_string(), "ethusdt@trade".to_string()];
        let id = 1;
        let message = unsubscribe_message(streams.clone(), id);
        let expected_message = json!({
            "method": "UNSUBSCRIBE",
            "params": streams,
            "id": id
        });
        let message_json: serde_json::Value = serde_json::from_str(&message).unwrap();
        assert_eq!(message_json, expected_message);
    }
}
