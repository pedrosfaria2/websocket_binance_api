mod menu;
mod storage;
mod subscription;
mod ui;
mod websocket;

use menu::show_menu;

#[tokio::main]
async fn main() {
    let symbols = subscription::fetch_symbols().await;
    match symbols {
        Ok(symbols) => show_menu(&symbols).await,
        Err(e) => eprintln!("Error fetching symbols: {}", e),
    }
}
