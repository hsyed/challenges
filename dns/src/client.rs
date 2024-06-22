use std::io::Result;

use tokio::net::UdpSocket;

use super::protocol::Message;

pub struct DnsClient {
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
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connect() {
        let client = DnsClient::connect("8.8.8.8:53").await.unwrap();
        let sample = [15, 245, 1, 32, 0, 1, 0, 0, 0, 0, 0, 1, 3, 119, 119, 119, 6, 103, 111, 111, 103, 108, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1, 0, 0, 41, 16, 0, 0, 0, 0, 0, 0, 0];
        let message = Message::from_bytes(&sample).unwrap();
        let res = client.query(&message).await.unwrap();
        println!("{:?}", res)
    }
}