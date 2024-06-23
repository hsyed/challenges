use std::io::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use super::protocol::Message;

///. DnsClient is a simple DNS client that sends a query to a DNS server and waits for a response.
struct DnsClient {
    socket: UdpSocket,
}

impl DnsClient {
    pub async fn connect(addr: &str) -> Result<DnsClient> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(addr).await?;
        Ok(DnsClient { socket })
    }

    pub async fn query(&self, msg: &Message) -> Result<Box<Message>> {
        let packet = msg.to_udp_packet()?;
        self.socket.send(packet.as_slice()).await?;
        let mut buf = [0; 4096];
        let len = self.socket.recv(&mut buf).await?;
        Message::from_bytes(&buf[..len])
    }
}

#[cfg(test)]
mod client_tests {
    use super::*;

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

/// Processor is the main struct that listens for incoming DNS queries and forwards them to a
/// DNS server.
pub struct Processor {
    ctx: Arc<Context>,
}

/// Context is a struct that holds the processing state of the Processor.
struct Context {
    socket: UdpSocket,
    client: DnsClient,
}

impl Processor {
    pub async fn build() -> Result<Processor> {
        Ok(
            Processor {
                ctx: Arc::new(
                    Context {
                        socket: UdpSocket::bind("127.0.0.1:1053").
                            await.expect("couldn't bind to address"),
                        client: DnsClient::connect("8.8.8.8:53")
                            .await.expect("couldn't connect forwarder"),
                    }
                )
            }
        )
    }

    pub async fn run_loop(&self) {
        loop {
            let mut buf = [0; 4096];
            match self.ctx.socket.recv_from(&mut buf).await {
                Ok((amt, src)) =>
                    self.handle_packet(&buf[..amt], src),
                Err(e) => {
                    // TODO this should probably take down the server or a watchdog should be
                    // trying to re-establish the socket ?
                    eprintln!("couldn't receive a datagram: {}", e); // TODO: where does eprintln send this ?
                }
            }
        }
    }

    fn handle_packet(&self, buf: &[u8], src: SocketAddr) {
        // println!("Data: {:?}", &buf[..amt]);
        // println!("Received {} bytes from {}", amt, src);
        match Message::from_bytes(buf) {
            Ok(query) => {
                let ctx = self.ctx.clone();
                tokio::spawn(async move {
                    // Todo return error to client
                    println!("Query: {:?}", query);
                    let res = ctx.client.query(&query)
                        .await.unwrap();
                    println!("Res: {:?}", res);
                    ctx.socket.send_to(res.to_udp_packet().unwrap().as_slice(), &src)
                        .await.unwrap();
                });
            }
            Err(e) => {
                eprintln!("Error parsing query: {:?}", e); // TODO return an error the client
            }
        };
    }
}
