pub mod pkts;
mod remote_client;

use crate::quit::Quit;
use connect::{tcp::TcpListener, StreamExt};
use log::*;
use remote_client::{Client, ClientId};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::{sync::Mutex as AsyncMutex, task};

pub struct ServerData {
    clients_by_id: RwLock<HashMap<ClientId, Arc<Client>>>,
    clients_by_name: RwLock<HashMap<String, Arc<Client>>>,
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
                clients_by_id: RwLock::new(HashMap::new()),
                clients_by_name: RwLock::new(HashMap::new()),
            }),
        }
    }

    pub async fn run(mut self) {
        let mut next_id: ClientId = 0;
        let mut quit = self.quit.subscribe();

        loop {
            tokio::select! {
                conn = self.listener.next() => {
                    let conn = conn.expect("Listener interrupted");

                    info!("Client from {} assigned id {next_id}", conn.peer_addr());

                    let (reader, writer) = conn.split();
                    let client = Arc::new(Client {
                        id: next_id,
                        conn: AsyncMutex::new(writer),
                        server: Arc::downgrade(&self.data),
                        quit: self.quit.clone(),
                    });

                    self.data
                        .clients_by_id
                        .write()
                        .unwrap()
                        .insert(next_id, Arc::clone(&client));

                    next_id += 1;

                    task::spawn(async move { (*client).run(reader).await });
                },
                _ = quit.recv() => {
                    trace!("Quit signal received");
                    break;
                },
                else => unreachable!("Quit channel broke"),
            }
        }

        info!("Stopped server");
    }
}
