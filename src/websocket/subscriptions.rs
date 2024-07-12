pub fn subscribe_message(streams: Vec<String>, id: u64) -> String {
    serde_json::json!({
        "method": "SUBSCRIBE",
        "params": streams,
        "id": id
    })
    .to_string()
}

pub fn unsubscribe_message(streams: Vec<String>, id: u64) -> String {
    serde_json::json!({
        "method": "UNSUBSCRIBE",
        "params": streams,
        "id": id
    })
    .to_string()
}
