use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result};
use std::sync::Arc;
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::sync::{Mutex, oneshot};
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};

use super::protocol::Message;

/// Slots tracks that state to support de-multiplexing responses.
struct Slots {
    pending: HashMap<u16, (u16, oneshot::Sender<Result<Message>>)>,
    counter: u16,
}

impl Slots {
    fn new() -> Slots {
        Slots {
            pending: HashMap::new(),
            counter: 0,
        }
    }

    fn create(&mut self, orig_id: u16) -> Result<(u16, oneshot::Receiver<Result<Message>>)> {
        if self.pending.len() == ((u16::MAX as usize) +1) {
            return Err(Error::new(ErrorKind::Other, "out of slots"))
        }

        let (tx, rx) = oneshot::channel();
        // find a free key
        self.counter = self.counter.wrapping_add(1);
        while self.pending.contains_key(&self.counter) {
            self.counter = self.counter.wrapping_add(1);
        }

        let client_id = self.counter;
        self.pending.insert(client_id, (orig_id, tx));
        Ok((client_id, rx))
    }

    fn remove(&mut self, id: u16) -> Option<(u16, oneshot::Sender<Result<Message>>)> {
        self.pending.remove(&id)
    }
}

struct Channel {
    socket: UdpSocket,
    addr: String,
    slots: Mutex<Slots>,
}

pub struct DnsClient {
    st: Arc<Channel>,
    r_handle: JoinHandle<()>,
}

const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

impl DnsClient {
    pub async fn connect(addr: &str) -> Result<DnsClient> {
        let socket = UdpSocket::bind("0.0.0.0:0").await.expect("couldn't bind");

        let st = Arc::new(Channel {
            socket,
            addr: String::from(addr),
            slots: Mutex::new(Slots::new()),
        });

        let r_handle = Self::start_receive_loop(st.clone());

        Ok(DnsClient { st, r_handle })
    }

    fn start_receive_loop(st: Arc<Channel>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut buf = [0; 4096];
            loop {
                match st.socket.recv_from(&mut buf).await {
                    Ok((len, _)) => {
                        match Message::from_bytes(&buf[..len]) {
                            Ok(mut msg) => {
                                if let Some((o_id, tx)) = st.slots.lock().await.remove(msg.header.id) {
                                    msg.header.id = o_id;
                                    if let Err(e) = tx.send(Ok(msg)) {
                                        eprintln!("err demul send: {:?}", e);
                                        continue
                                    }
                                } else {
                                    eprintln!("dns client received orphaned msg: {:?}", msg);
                                    continue
                                }
                            }
                            Err(e) => {
                                eprintln!("malformed packet: {}\ndata: {:?}",e, &buf[..len]);
                                continue
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("failed on socket receive: {}", e);
                        sleep(Duration::from_millis(100)).await;
                        continue;
                    }
                }
            }
        })
    }

    pub async fn query(&self, msg: &Message) -> Result<Message> {
        let (client_id, rx) = self.st.slots.lock().await.create(msg.header.id)?;
        let packet = msg.to_udp_packet(Some(client_id)).unwrap();
        if let Err(e) = self.st.socket.send_to(packet.as_slice(), &self.st.addr).await {
            self.st.slots.lock().await.remove(client_id);
            return Err(e);
        }

        match timeout(CLIENT_TIMEOUT, rx).await {
            Ok(rcv) => {
                match rcv {
                    Ok(res) => res,
                    Err(e) => {
                        self.st.slots.lock().await.remove(client_id);
                        Err(Error::new(ErrorKind::TimedOut, e))
                    }
                }
            }
            Err(e) => {
                self.st.slots.lock().await.remove(client_id);
                Err(Error::new(ErrorKind::TimedOut, e))
            }
        }
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
