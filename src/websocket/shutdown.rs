use tokio::sync::mpsc::Sender;

pub async fn handle_shutdown(shutdown_tx: Sender<()>) {
    // Wait for Ctrl+C signal
    tokio::signal::ctrl_c().await.unwrap();
    // Send shutdown signal
    let _ = shutdown_tx.send(()).await;
}
