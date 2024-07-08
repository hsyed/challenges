use std::sync::Arc;

use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter, Result};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

use crate::protocol::{Command, RetrievalCommand, StorageCommand, StorageCommandType, Value};

#[derive(Debug)]
pub(crate) struct Connection {
    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>,
    buffer: Vec<u8>,
}

impl Connection {
    pub(crate) fn new(stream: TcpStream) -> Connection {
        let (reader, writer) = stream.into_split();
        Connection {
            reader: BufReader::new(reader),
            writer: BufWriter::new(writer),
            buffer: Vec::with_capacity(512),
        }
    }

    pub(crate) async fn read_command(&mut self) -> Result<Command> {
        read_command(&mut self.reader, &mut self.buffer).await
    }

    pub(crate) async fn write_value(&mut self, key: &String, val: Arc<Value>) -> Result<()> {
        self.writer.write_all(b"VALUE ").await?;
        self.writer.write_all(key.as_bytes()).await?;
        self.writer.write_all(format!(" {} {}\r\n", val.flags, val.data.len()).as_bytes()).await?;
        self.writer.write_all(&val.data).await?;
        self.writer.write_all(b"\r\n").await?;
        Ok(())
    }

    pub(crate) async fn write_response(&mut self, bytes: &[u8]) -> Result<()> {
        self.writer.write_all(bytes).await?;
        self.writer.write_all(b"\r\n").await?;
        self.writer.flush().await?;
        Ok(())
    }
}

async fn read_command<R: AsyncBufRead + Unpin>(r: &mut R, buf: &mut Vec<u8>) -> Result<Command> {
    buf.clear();
    let len = r.read_until(b'\n', buf).await?;
    let buf = &buf[..len];
    if &buf[len - 2..] != b"\r\n" {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "malformed command"));
    }
    match parse_partial_command(&buf[..len - 2])? {
        Command::Storage(mut com) => {
            let mut data = vec![0; com.byte_count as usize];
            r.read_exact(&mut data).await?;
            let mut terminal = [0u8; 2];
            r.read_exact(&mut terminal).await?;
            if &terminal != b"\r\n" {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "malformed command"));
            }
            com.data = data;
            Ok(Command::Storage(com))
        }
        other => Ok(other),
    }
}

/// parse a partial command,
fn parse_partial_command(command_line: &[u8]) -> Result<Command> {
    let mut parts = command_line.split(|&b| b == b' ').filter(|part| !part.is_empty());

    let command = parts.next().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing command"))?;
    let key = parts.next().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing key"))?;
    if key.len() > 250 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "key too long"));
    }
    let key = std::str::from_utf8(key).map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid key"))?;

    if command == b"get" {
        if parts.next().is_some() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "malformed get command"));
        }
        return Ok(Command::Retrieval(RetrievalCommand::Get { key: key.to_string() }));
    }

    let st_command_type = StorageCommandType::from_bytes(command)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "unrecognised command"))?;

    let mut read_int = |field_id: &str| -> std::io::Result<u32> {
        let value = parts.next().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("missing {}", field_id)))?;
        let value = std::str::from_utf8(value).map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid {}", field_id)))?;
        value.parse().map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid {}", field_id)))
    };

    let flags = read_int("flags")?;
    let exptime = read_int("exptime")?;
    let byte_count = read_int("bytes")?;

    let no_reply: bool = match parts.next() {
        Some(b"noreply") => true,
        None => false,
        Some(x) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("malformed command: {:?}", std::str::from_utf8(x)))),
    };
    Ok(
        Command::Storage(
            StorageCommand {
                command: st_command_type,
                no_reply,
                byte_count,
                flags,
                key: key.to_string(),
                exp_time: exptime,
                data: Vec::new(),
            }
        )
    )
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use tokio::io::BufReader;

    use crate::connection::{parse_partial_command, read_command};
    use crate::protocol::{Command, StorageCommandType};

    #[test]
    fn test_parse_partial_command() {
        let res = parse_partial_command(b"set key 0 60 4").unwrap();
        match res {
            Command::Storage(com) => {
                assert_eq!(com.command, StorageCommandType::Set);
                assert_eq!(com.key, "key");
                assert_eq!(com.flags, 0);
                assert_eq!(com.exp_time, 60);
                assert_eq!(com.no_reply, false);
                assert_eq!(com.byte_count, 4);
            }
            _ => panic!()
        }
        ()
    }

    #[tokio::test]
    async fn test_read_command() -> std::io::Result<()> {
        let cursor = Cursor::new(b"set key 0 60 5\r\nvalue\r\n");
        let mut br = BufReader::new(cursor);
        let mut vec = Vec::new();
        let res = read_command(&mut br, &mut vec).await?;
        println!("{:?}", res);
        Ok(())
    }
}