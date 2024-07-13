mod menu;
mod storage;
mod subscription;
mod ui;
mod websocket;

use menu::show_menu;

#[tokio::main]
async fn main() {
    show_menu().await;
}
