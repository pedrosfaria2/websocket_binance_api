use tokio::sync::oneshot::Sender;

pub async fn handle_shutdown(shutdown_tx: Sender<()>) {
    tokio::signal::ctrl_c().await.unwrap();
    let _ = shutdown_tx.send(());
}
