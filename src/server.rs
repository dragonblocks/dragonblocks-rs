#![allow(unused)]

pub mod pkts;

use super::pkt;
use super::quit::Quit;
use connect::{tcp::TcpListener, ConnectDatagram, ConnectionReader, ConnectionWriter, StreamExt};
use pkts::*;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, Weak},
};
use tokio::{
    sync::{broadcast, mpsc, Mutex as AsyncMutex},
    task,
};

type ClientId = u64;

pub struct Client {
    pub id: ClientId,
    pub conn: AsyncMutex<ConnectionWriter>,
    pub server: Weak<ServerData>,
    quit: Quit,
}

impl Client {
    async fn login(&self, pkt: &Login) {
        println!("login {} {}", pkt.name, pkt.pwd);
    }

    async fn run(&self, mut reader: ConnectionReader) {
        let mut quit = self.quit.subscribe();

        loop {
            tokio::select! {
                Some(msg) = reader.next() => match msg.recipient() {
                    LOGIN if let Some(pkt) = pkt::get::<Login>(&msg) =>
                        self.login(&pkt).await,
                    _ => {},
                },
                _ = quit.recv() => break,
                else => break,
            }
        }

        if let Some(server) = self.server.upgrade() {
            server.clients_by_id.write().unwrap().remove(&self.id);
        }

        println!("disconnect {}", self.id);
    }
}

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
        println!("listen {addr}");

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
                Some(conn) = self.listener.next() => {
                    println!("connect {}", conn.peer_addr());

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
                _ = quit.recv() => break,
                else => break,
            }
        }

        println!("shutdown");
    }
}
