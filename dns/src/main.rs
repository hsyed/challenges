use std::io::Result;
use std::sync::Arc;

use tokio::net::UdpSocket;

use client::DnsClient;
use protocol::Message;

mod protocol;
mod client;

#[tokio::main]
async fn main() {
    let ctx = ProcessingContext::setup().
        await.expect("could not startup");
    ProcessingContext::run(ctx).await
}

struct ProcessingContext {
    socket: UdpSocket,
    client: DnsClient,
}

impl ProcessingContext {
    async fn setup() -> Result<Arc<ProcessingContext>> {
        Ok(Arc::new(ProcessingContext {
            socket: UdpSocket::bind("127.0.0.1:1053").
                await.expect("couldn't bind to address"),
            client: DnsClient::connect("8.8.8.8:53").await?,
        }))
    }

    async fn run(ctx: Arc<ProcessingContext>) {
        // RFC: the outer dispatch loop could be thread-per-core ?
        loop {
            let mut buf = [0; 4096];
            match ctx.socket.recv_from(&mut buf).await {
                Ok((amt, src)) => {
                    // println!("Data: {:?}", &buf[..amt]);
                    // println!("Received {} bytes from {}", amt, src);
                    match Message::from_bytes(&buf) {
                        Ok(query) => {
                            let ctx = ctx.clone();
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
                Err(e) => {
                    eprintln!("couldn't receive a datagram: {}", e); // TODO: where does eprintln send this ?
                }
            }
        }
    }
}
