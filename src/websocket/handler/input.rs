use crossterm::event::{self, Event as CEvent, KeyCode};
use tokio::sync::mpsc;

pub async fn handle_input(shutdown_tx: mpsc::Sender<()>) {
    loop {
        // Poll for events every 10 milliseconds
        if event::poll(std::time::Duration::from_millis(10)).unwrap() {
            // Read the event
            if let CEvent::Key(key) = event::read().unwrap() {
                // Check if the 'q' key is pressed
                if key.code == KeyCode::Char('q') {
                    // Send shutdown signal
                    shutdown_tx
                        .send(())
                        .await
                        .expect("failed to send shutdown signal");
                    break;
                }
            }
        }
    }
}
