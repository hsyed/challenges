use std::io::Result;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::UdpSocket;
use crate::client::DnsClient;

use super::cache::DnsCache;
use super::protocol::{Message, ResourceRecord};

/// Context is a struct that holds the processing state of the Processor.
struct Context {
    socket: UdpSocket,
    client: DnsClient,
    cache: DnsCache,
}

/// Processor is the main struct that listens for incoming DNS queries and forwards them to a
/// DNS server.
pub struct Processor {
    ctx: Arc<Context>,
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

    async fn handle_query(src: &SocketAddr, query: &Message, ctx: &Arc<Context>) {
        // Todo validate the query
        // Todo add cache hit/miss metrics
        println!("Query: {:?}", query);
        if query.questions.len() == 1 {
            if let Some(answers) = ctx.cache.get(&query.questions[0]).await {
                Self::respond_from_cache(&src, query, ctx, answers).await;
                return;
            } else {
                Self::do_query(src, query, ctx, true).await;
                return
            }
        } else { // more than one question -- we just pass that through
            Self::do_query(src, query, ctx, false).await;
            return;
        }
    }

    async fn respond_from_cache(src: &SocketAddr, query: &Message, ctx: &Arc<Context>, answers: Vec<ResourceRecord>) {
        println!("from cache");
        let mut response = query.clone();
        response.header.flags.set_qr(1);
        response.header.ancount = answers.len() as u16;
        response.answers = answers.clone();
        let _ = ctx.socket.send_to(response.to_udp_packet(None).unwrap().as_slice(), &src).await;
    }

    async fn do_query(src: &SocketAddr, query: &Message, ctx: &Arc<Context>, set_cache: bool) {
        match ctx.client.query(query).await {
            Ok(res) => {
                if set_cache {
                    ctx.cache.set(&query.questions[0], &res.answers).await;
                }
                let packet = res.to_udp_packet(None).unwrap();
                let _ = ctx.socket.send_to(packet.as_slice(), &src).await; // TODO handle error
                return
            }
            Err(_) => {
                let mut response = query.clone();
                response.header.flags.set_qr(1);
                response.header.flags.set_rcode(2); // Server failure
                let packet = response.to_udp_packet(None).unwrap();
                let _ = ctx.socket.send_to(packet.as_slice(), &src).await;
                return
            }
        }
    }
}
