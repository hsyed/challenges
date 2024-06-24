use std::io::Result;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::UdpSocket;
use crate::client::DnsClient;

use super::cache::DnsCache;
use super::protocol::Message;

/// Processor is the main struct that listens for incoming DNS queries and forwards them to a
/// DNS server.
pub struct Processor {
    ctx: Arc<Context>,
}

/// Context is a struct that holds the processing state of the Processor.
struct Context {
    socket: UdpSocket,
    client: DnsClient,
    cache: DnsCache,
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
                        cache: DnsCache::new(),
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
                    Self::handle_query(&src, &query, &ctx).await;
                });
            }
            Err(e) => {
                eprintln!("Error parsing query: {:?}", e); // TODO return an error the client
            }
        };
    }

    async fn handle_query(src: &SocketAddr, query: &Box<Message>, ctx: &Arc<Context>) {
        // Todo validate the query
        // Todo return error to client
        // Todo add cache hit/miss metrics
        println!("Query: {:?}", query);
        if query.questions.len() == 1 {
            if let Some(answers) = ctx.cache.get(&query.questions[0]).await {
                println!("from cache");
                let mut response = query.clone();
                response.header.flags.set_qr(1);
                response.header.ancount = answers.len() as u16;
                response.answers = answers.clone();
                ctx.socket.send_to(response.to_udp_packet(None).unwrap().as_slice(), &src)
                    .await.unwrap();
                return;
            } else {
                let res = ctx.client.query(query).await.unwrap(); // TODO must handle error here
                ctx.cache.set(&query.questions[0], &res.answers).await;
                ctx.socket.send_to(res.to_udp_packet(None).unwrap().as_slice(), &src)
                    .await.unwrap();
                return;
            }
        } else { // more than one question -- we just pass that through
            let res = ctx.client.query(query).await.unwrap();
            ctx.socket.send_to(res.to_udp_packet(None).unwrap().as_slice(), &src)
                .await.unwrap();
            return;
        }
    }
}
