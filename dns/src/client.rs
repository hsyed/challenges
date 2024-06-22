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

    pub fn query(&self, _msg: &Message) -> Result<Message> {
        todo!("send query to server and return response")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connect() {

    }
}