use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{Mutex, oneshot};
use std::io;
use tokio::net::UdpSocket;
use tokio::task::JoinHandle;
use crate::protocol::Message;

/// Slots tracks that state to support de-multiplexing responses.
struct Slots {
    pending: HashMap<u16, (u16, oneshot::Sender<io::Result<Box<Message>>>)>,
    counter: u16,
}

impl Slots {
    fn new() -> Slots {
        Slots {
            pending: HashMap::new(),
            counter: 0,
        }
    }

    fn create(&mut self, orig_id: u16) -> (u16, oneshot::Receiver<io::Result<Box<Message>>>) {
        let (tx, rx) = oneshot::channel();
        // find a free key
        self.counter = self.counter.wrapping_add(1);
        while self.pending.contains_key(&self.counter) { // todo prevent infinite loop somehow ?
            self.counter = self.counter.wrapping_add(1);
        }

        let client_id = self.counter;
        self.pending.insert(client_id, (orig_id, tx));
        (client_id, rx)
    }

    fn remove(&mut self, id: u16) -> Option<(u16, oneshot::Sender<io::Result<Box<Message>>>)> {
        self.pending.remove(&id)
    }
}

struct State {
    socket: UdpSocket,
    slots: Mutex<Slots>,
}

pub struct DnsClient {
    st: Arc<State>,
    r_handle: JoinHandle<()>,
}

impl DnsClient {
    pub async fn connect(addr: &str) -> io::Result<DnsClient> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(addr).await?;

        let st = Arc::new(State {
            socket,
            slots: Mutex::new(Slots {
                pending: HashMap::new(),
                counter: 0,
            }),
        });

        let r_handle = Self::start_sync_loop(st.clone());

        Ok(DnsClient { st, r_handle })
    }

    // TODO Currently once the sync loop fails:
    //   1. new requests will fail on socket.send.
    //   2. the sync loop will fail to receive and terminate.
    //
    // The client should probably transparently re-establish the comms channel similar to the way
    // gRPC is implemented. Alternatively the client should reach a terminal state and the user
    // should be responsible for re-creating the DnsClient as the recovery mechanism.


    fn start_sync_loop(st: Arc<State>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut buf = [0; 4096];
            loop {
                let (len, _) = st.socket.recv_from(&mut buf).await.unwrap();
                let mut msg = Message::from_bytes(&buf[..len]).unwrap();
                if let Some((orig_id, tx)) = st.slots.lock().await.remove(msg.header.id) {
                    msg.header.id = orig_id;
                    tx.send(Ok(msg)).unwrap();
                } else {
                    eprintln!("Received orphaned message with id: {}", msg.header.id);
                }
            }
        })
    }

    pub async fn query(&self, msg: &Box<Message>) -> io::Result<Box<Message>> {
        let (client_id, rx) = self.st.slots.lock().await.create(msg.header.id);
        self.st.socket.send(msg.to_udp_packet(Some(client_id)).unwrap().as_slice()).await?;
        // TODO if the send fails the slot should be removed.

        // TODO flesh out timeout, orphaned slots need to be cleared out.
        // match timeout(Duration::from_secs(30), rx).await {
        //    Ok(res) => match res {
        //         Ok(res) => Ok(res),
        //         Err(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "timeout")),
        //       },
        //     Err(_) => {
        //         let mut slots = self.st.slots.lock().await;
        //         slots.pending.remove(&client_id);
        //         Err(std::io::Error::new(std::io::ErrorKind::Other, "timeout")
        //    }
        // }
        rx.await.unwrap()
    }
}

impl Drop for DnsClient {
    fn drop(&mut self) {
        self.r_handle.abort();
    }
}


#[cfg(test)]
mod client_tests {
    use crate::client::DnsClient;
    use crate::protocol::Message;

    // TODO distinguish "manual" tests from unit tests.
    #[tokio::test]
    async fn test_connect() {
        let client = DnsClient::connect("8.8.8.8:53").await.unwrap();
        let sample = [15, 245, 1, 32, 0, 1, 0, 0, 0, 0, 0, 1, 3, 119, 119, 119, 6, 103, 111, 111, 103, 108, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1, 0, 0, 41, 16, 0, 0, 0, 0, 0, 0, 0];
        let message = Message::from_bytes(&sample).unwrap();
        let res = client.query(&message).await.unwrap();
        println!("{:?}", res)
    }
}
