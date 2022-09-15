use connect::{ConnectDatagram, ConnectionWriter, SinkExt};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as AsyncMutex;

pub fn get<'a, T: Deserialize<'a>>(pkt: &'a ConnectDatagram) -> Option<T> {
    serde_cbor::from_slice(pkt.data()?).ok()
}

pub async fn put<T: Serialize>(conn: &AsyncMutex<ConnectionWriter>, id: u16, pkt: &T) {
    conn.lock()
        .await
        .send(ConnectDatagram::new(id, serde_cbor::to_vec(pkt).unwrap()).unwrap())
        .await
        .unwrap();
}
