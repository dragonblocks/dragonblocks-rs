use log::*;
use tokio::sync::{broadcast, mpsc};

#[derive(Clone)]
pub struct Quit {
    signal: broadcast::Sender<()>,

    #[allow(unused)]
    confirm: mpsc::Sender<()>,
}

impl Quit {
    pub fn new() -> (Self, mpsc::Receiver<()>) {
        let (confirm, confirm_recv) = mpsc::channel(1);
        let (signal, _) = broadcast::channel(1);

        (Self { confirm, signal }, confirm_recv)
    }

    pub fn quit(&self) {
        info!("Shutting down");
        self.signal.send(()).unwrap();
    }

    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.signal.subscribe()
    }
}
