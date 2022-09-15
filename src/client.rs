use super::pkt;
use super::quit::Quit;
use super::server::pkts as server_bound;
use connect::{Connection, ConnectionReader, ConnectionWriter, StreamExt};
use tokio::sync::Mutex as AsyncMutex;

pub struct Client {
    pub conn: AsyncMutex<ConnectionWriter>,
    quit: Quit,
}

impl Client {
    pub async fn run(addr: &str, quit: Quit) {
        let (reader, writer) = Connection::tcp_client(addr).await.unwrap().split();

        println!("client connect {addr}");
        Self {
            conn: AsyncMutex::new(writer),
            quit,
        }
        .run_loop(reader)
        .await;
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
                Some(msg) = reader.next() => {
                    println!("{}", msg.recipient());
                },
                _ = quit.recv() => break,
                else => break,
            }
        }

        println!("client disconnect");
    }
}
