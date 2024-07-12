mod menu;
mod subscription;
mod websocket;

#[tokio::main]
async fn main() {
    menu::show_menu().await;
}
