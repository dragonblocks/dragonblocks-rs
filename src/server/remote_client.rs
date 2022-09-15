use crate::{
    pkt,
    quit::Quit,
    server::{pkts::*, ServerData},
};
use connect::{ConnectDatagram, Connection, ConnectionReader, ConnectionWriter, StreamExt};
use log::*;
use once_cell::sync::OnceCell;
use std::{
    fmt,
    net::SocketAddr,
    sync::{Arc, Weak},
};
use tokio::{sync::Mutex as AsyncMutex, task};

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct Client {
    addr: SocketAddr,
    data: OnceCell<ClientData>,

    #[allow(unused)]
    #[derivative(Debug = "ignore")]
    conn: AsyncMutex<ConnectionWriter>,

    #[derivative(Debug = "ignore")]
    server: Weak<ServerData>,

    #[derivative(Debug = "ignore")]
    circle: OnceCell<Weak<Client>>,

    #[derivative(Debug = "ignore")]
    quit: Quit,
}

#[derive(Debug)]
pub struct ClientData {
    name: String,
}

impl Client {
    pub async fn create(conn: Connection, server: Weak<ServerData>, quit: Quit) {
        let addr = conn.peer_addr();
        let (reader, writer) = conn.split();

        let client = Arc::new(Self {
            addr,
            server,
            quit,
            conn: AsyncMutex::new(writer),
            data: OnceCell::new(),
            circle: OnceCell::new(),
        });

        client
            .circle
            .set(Arc::downgrade(&client))
            .expect("OnceCell was just created");

        task::spawn(async move {
            (*client).run(reader).await;
        });
    }

    async fn login(&self, pkt: &Login) {
        if let None = self.data.get() && let Some(server) = self.server.upgrade() {
			server
				.players
				.write()
				.expect("deadlock")
				.entry(pkt.name.clone())
				.or_insert_with(|| {
					self.data.set(ClientData {
						name: pkt.name.clone(),
					}).expect("OnceCell was verified to be empty above");

					info!("{self}: logged as {}", pkt.name);

					self.circle
						.get()
						.expect("OnceCell was initialized in fn create")
						.upgrade()
						.expect("self can't be dropped")
				});
		}
    }

    async fn handle(&self, msg: &ConnectDatagram) {
        match msg.recipient() {
			LOGIN if let Some(pkt) = pkt::get::<Login>(msg) => self.login(&pkt).await,
			_ => warn!("{self}: invalid packet with recipient {}", msg.recipient()),
		}
    }

    async fn run(&self, mut reader: ConnectionReader) {
        info!("{self}: connected");

        let mut quit = self.quit.subscribe();

        loop {
            tokio::select! {
                msg = reader.next() => match msg {
                    Some(msg) => self.handle(&msg).await,
                    None => {
                        trace!("{self}: closed connection");
                        break;
                    }
                },
                _ = quit.recv() => {
                    trace!("{self}: quit signal received");
                    break;
                },
                else => unreachable!("quit channel broke"),
            }
        }

        if let Some(data) = self.data.get() && let Some(srv) = self.server.upgrade() {
			srv.players.write().expect("deadlock").remove(&data.name);

			trace!("{self}: removed from clients");
		}

        info!("{self}: disconnected");
    }
}

impl fmt::Display for Client {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.addr)
    }
}
