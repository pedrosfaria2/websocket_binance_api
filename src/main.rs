mod menu;
mod subscription;
mod websocket;
mod storage;

#[tokio::main]
async fn main() {
    menu::show_menu().await;
}
