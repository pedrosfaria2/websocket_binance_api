mod websocket;
mod storage;
mod menu;
mod ui;
mod subscription;

use menu::show_menu;

#[tokio::main]
async fn main() {
    show_menu().await;
}
