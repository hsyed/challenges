use tokio::net::UdpSocket;
use protocol::Message;
use crate::client::DnsClient;


mod protocol;
mod client;

struct handler {
    client: DnsClient,
}

impl handler {
    async fn new() -> handler { // TODO fix this, it shouldn't await here probs ?
        handler {
            client: DnsClient::connect("8.8.8.8:53")
                .await
                .expect("could not connect to server"),
        }
    }

    async fn handle(&self, query: &Message) -> std::io::Result<Message> {
        self.client.query(query).await
    }
}

// what I would like is for the messages to come off the socket as fast as possible with a dedicated
// thread -- could be thread-per-core.

// I'd like to then hand off the DNS messages to a bounded concurrency processing queue. This queue
// processor becomes responsible for supervising the lifecycle of the requests. Process:
//   * forward the message to the Google DNS servers
//   * return the response message to the client (verbatim I think?)
//   * handle IO errors
//   * handle timeouts
//   * consult a cache
//
// In addition to the message itself, the processing queue will need to know:
//   * The socket
//   * The ephemeral address of the client

// Questions:
//   What should the cache key be ? The question bytes could maybe be used verbatim ?
#[tokio::main]
async fn main() {
    run_async().await
}

async fn run_async() {
    let handler = &handler::new().await;

    let socket = UdpSocket::bind("127.0.0.1:1053").
        await.expect("could not bind to address");

    // RFC: this dispatch loop could be thread-per-core ?
    loop {
        let mut buf = [0; 4096];
        match socket.recv_from(&mut buf).await {
            Ok((amt, src)) => {
                println!("Data: {:?}", &buf[..amt]);
                println!("Received {} bytes from {}", amt, src);
                match Message::from_bytes(&buf) {
                    Ok(query) => {
                        println!("{:?}", query);
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
