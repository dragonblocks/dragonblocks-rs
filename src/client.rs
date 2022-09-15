use crate::pkt;
use crate::quit::Quit;
use crate::server::pkts as server_bound;
use connect::{ConnectDatagram, Connection, ConnectionReader, ConnectionWriter, StreamExt};
use log::*;
use tokio::sync::Mutex as AsyncMutex;

pub struct Client {
    pub conn: AsyncMutex<ConnectionWriter>,
    quit: Quit,
}

impl Client {
    pub async fn run(addr: &str, quit: Quit) {
        let (reader, writer) = Connection::tcp_client(addr).await.unwrap().split();

        Self {
            conn: AsyncMutex::new(writer),
            quit,
        }
        .run_loop(reader)
        .await;
    }

    async fn handle(&self, msg: &ConnectDatagram) {
        println!("{}", msg.recipient());
    }

    async fn run_loop(self, mut reader: ConnectionReader) {
        let mut quit = self.quit.subscribe();

        pkt::put(
            &self.conn,
            server_bound::LOGIN,
            &server_bound::Login {
                name: "player".into(),
                pwd: "password123".into(),
            },
        )
        .await;

        loop {
            tokio::select! {
                msg = reader.next() => match msg {
                    Some(msg) => self.handle(&msg).await,
                    None => {
                        trace!("Server closed connection");
                        break;
                    }
                },
                _ = quit.recv() => {
                    trace!("Quit signal received");
                    break;
                },
                else => unreachable!("Quit channel broke"),
            }
        }

        info!("Disconnected");
    }
}
