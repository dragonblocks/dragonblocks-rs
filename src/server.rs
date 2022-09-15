pub mod pkts;
mod remote_client;

use crate::quit::Quit;
use connect::{tcp::TcpListener, StreamExt};
use log::*;
use remote_client::Client;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
pub struct ServerData {
    players: RwLock<HashMap<String, Arc<Client>>>,
}

pub struct Server {
    listener: TcpListener,
    data: Arc<ServerData>,
    quit: Quit,
}

impl Server {
    pub async fn new(addr: &str, quit: Quit) -> Self {
        Self {
            quit,
            listener: TcpListener::bind(addr).await.unwrap(),
            data: Arc::new(ServerData {
                players: RwLock::new(HashMap::new()),
            }),
        }
    }

    pub async fn run(mut self) {
        let mut quit = self.quit.subscribe();

        loop {
            tokio::select! {
                conn = self.listener.next() => Client::create(
                        conn.expect("listener interrupted"),
                        Arc::downgrade(&self.data),
                        self.quit.clone()
                    ).await,
                _ = quit.recv() => {
                    trace!("quit signal received");
                    break;
                },
                else => unreachable!("quit channel broke"),
            }
        }

        info!("stopped server");
    }
}
