use crossterm::event::{self, Event as CEvent, KeyCode};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub fn handle_input(shutdown_tx: mpsc::Sender<()>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            if event::poll(std::time::Duration::from_millis(10)).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    if key.code == KeyCode::Char('q') {
                        shutdown_tx.send(()).await.expect("failed to send shutdown signal");
                        break;
                    }
                }
            }
        }
    })
}
