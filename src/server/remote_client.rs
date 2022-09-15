use super::pkts::*;
use super::ServerData;
use crate::pkt;
use crate::Quit;
use connect::{ConnectDatagram, ConnectionReader, ConnectionWriter, StreamExt};
use log::*;
use std::sync::Weak;
use tokio::sync::Mutex as AsyncMutex;

pub type ClientId = u64;

pub struct Client {
    pub id: ClientId,
    pub conn: AsyncMutex<ConnectionWriter>,
    pub server: Weak<ServerData>,
    pub quit: Quit,
}

impl Client {
    async fn login(&self, pkt: &Login) {
        info!(
            "Client {id}: logged in {name} {pwd}",
            id = self.id,
            name = pkt.name,
            pwd = pkt.pwd
        );
    }

    async fn handle(&self, msg: &ConnectDatagram) {
        match msg.recipient() {
			LOGIN if let Some(pkt) = pkt::get::<Login>(msg) => self.login(&pkt).await,
			_ => warn!("Client {id}: Invalid packet with recipient {rep}",
				id = self.id, rep = msg.recipient()),
		}
    }

    pub async fn run(&self, mut reader: ConnectionReader) {
        let mut quit = self.quit.subscribe();

        loop {
            tokio::select! {
                msg = reader.next() => match msg {
                    Some(msg) => self.handle(&msg).await,
                    None => {
                        trace!("Client {id}: Closed connection", id = self.id);
                        break;
                    }
                },
                _ = quit.recv() => {
                    trace!("Client {id}: Quit signal received", id = self.id);
                    break;
                },
                else => unreachable!("Quit channel broke"),
            }
        }

        if let Some(server) = self.server.upgrade() {
            server.clients_by_id.write().unwrap().remove(&self.id);
            trace!("Client {id}: Removed from clients", id = self.id);
        }

        info!("Client {id}: Disconnected", id = self.id);
    }
}
