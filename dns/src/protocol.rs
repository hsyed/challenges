use std::io;
use std::io::{Cursor, Read, Write};
use std::io::Result;

#[derive(Debug)]
struct Header {
    id: u16,
    qr: u8,
    opcode: u8,
    aa: u8,
    tc: u8,
    rd: u8,
    ra: u8,
    z: u8,
    rcode: u8,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
}

impl Header {
    fn from_bytes(b: &[u8]) -> Header {
        Header {
            id: u16::from_be_bytes([b[0], b[1]]),
            qr: b[2] >> 7,
            opcode: (b[2] >> 3) & 0x0F,
            aa: (b[2] >> 2) & 0x01,
            tc: (b[2] >> 1) & 0x01,
            rd: b[2] & 0x01,
            ra: b[3] >> 7,
            z: (b[3] >> 4) & 0x07,
            rcode: b[3] & 0x0F,
            qdcount: u16::from_be_bytes([b[4], b[5]]),
            ancount: u16::from_be_bytes([b[6], b[7]]),
            nscount: u16::from_be_bytes([b[8], b[9]]),
            arcount: u16::from_be_bytes([b[10], b[11]]),
        }
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.id.to_be_bytes())?;
        writer.write_all(&[(self.qr << 7) | (self.opcode << 3) | (self.aa << 2) | (self.tc << 1) | self.rd])?;
        writer.write_all(&[(self.ra << 7) | (self.z << 4) | self.rcode])?;
        writer.write_all(&self.qdcount.to_be_bytes())?;
        writer.write_all(&self.ancount.to_be_bytes())?;
        writer.write_all(&self.nscount.to_be_bytes())?;
        writer.write_all(&self.arcount.to_be_bytes())?;
        Ok(())
    }
}

#[derive(Debug)]
struct Question {
    qname: String,
    qtype: u16,
    qclass: u16,
}

impl Question {
    fn read<R: Read>(r: &mut R) -> Result<Question> {
        Ok(
            Question {
                qname: read_labels_to_str(r)?,
                qtype: read_u16(r)?,
                qclass: read_u16(r)?,
            }
        )
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let qname = self.qname.split('.');
        for label in qname {
            writer.write_all(&[label.len() as u8])?;
            writer.write_all(label.as_bytes())?;
        }
        writer.write_all(&[0])?; // write the null byte
        writer.write_all(&self.qtype.to_be_bytes())?;
        writer.write_all(&self.qclass.to_be_bytes())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ResourceRecord {
    name: String,
    rtype: u16,
    rclass: u16,
    ttl: u32,
    rdlength: u16,
    rdata: Vec<u8>,
}

impl ResourceRecord {
    fn read<R: Read>(r: &mut R) -> Result<ResourceRecord> {
        let name = read_labels_to_str(r)?;
        let rtype = read_u16(r)?;
        let rclass = read_u16(r)?;
        let ttl = read_u32(r)?;
        let rdlength = read_u16(r)?;
        let mut rdata = vec![0; rdlength as usize];
        r.read_exact(&mut rdata)?;
        Ok(
            ResourceRecord {
                name,
                rtype,
                rclass,
                ttl,
                rdlength,
                rdata,
            }
        )
    }

    fn read_all<R: Read>(r: &mut R, count: u16) -> Result<Vec<ResourceRecord>> {
        let mut records = Vec::with_capacity(count as usize);
        for _ in 0..count {
            records.push(ResourceRecord::read(r)?);
        }
        Ok(records)
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let name = self.name.split('.');
        for label in name {
            writer.write_all(&[label.len() as u8])?;
            writer.write_all(label.as_bytes())?;
        }
        writer.write_all(&[0])?; // write the null byte
        writer.write_all(&self.rtype.to_be_bytes())?;
        writer.write_all(&self.rclass.to_be_bytes())?;
        writer.write_all(&self.ttl.to_be_bytes())?;
        writer.write_all(&self.rdlength.to_be_bytes())?;
        writer.write_all(&self.rdata)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Message {
    header: Header,
    questions: Vec<Question>,
    answers: Vec<ResourceRecord>,
    authorities: Vec<ResourceRecord>,
    additionals: Vec<ResourceRecord>,
}

impl Message {
    pub fn from_bytes(b: &[u8]) -> Result<Message> {
        let header = Header::from_bytes(b);
        let mut cur = Cursor::new(b);
        cur.set_position(12); // TODO consider using a cursor to read the header

        Ok(
            Message {
                questions: match header.qdcount {
                    0 => Vec::new(),
                    1 => vec!(Question::read(&mut cur)
                        .expect("could not parse question")),
                    _ => return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unsupported number of questions")
                    ),
                },
                answers: ResourceRecord::read_all(&mut cur, header.ancount)?,
                authorities: ResourceRecord::read_all(&mut cur, header.nscount)?,
                additionals: ResourceRecord::read_all(&mut cur, header.arcount)?,
                header,
            }
        )
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.header.write(writer)?;
        // TODO make conditional a) it being a query (?) and b) if a question is present ? or
        // should this business logic be added to a builder ?
        for question in &self.questions {
            question.write(writer)?;
        }
        for answer in &self.answers {
            answer.write(writer)?;
        }
        for authority in &self.authorities {
            authority.write(writer)?;
        }
        for additional in &self.additionals {
            additional.write(writer)?;
        }
        Ok(())
    }
}

fn read_u16<R: Read>(r: &mut R) -> Result<u16> {
    let mut buf = [0; 2];
    r.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

fn read_u32<R: Read>(r: &mut R) -> Result<u32> {
    let mut buf = [0; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

fn read_labels_to_str<R: Read>(r: &mut R) -> Result<String> {
    let mut qname = String::new();
    loop {
        let mut len = [0];
        r.read_exact(&mut len)?;
        if len[0] == 0 {
            break;
        }
        if !qname.is_empty() {
            qname.push('.');
        }
        let mut label = vec![0; len[0] as usize];
        r.read_exact(&mut label)?;
        qname.push_str(
            std::str::from_utf8(&label).expect("invalid utf8 label")
        );
    }
    Ok(qname)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_serialization() {
        // 0, 0, 41, 16, 0, 0, 0, 0, 0, 0, 0
        // in the sample below the trailing bytes represent the additional section.
        // signalling EDNS0 support.
        // https://datatracker.ietf.org/doc/html/rfc6891
        // first 0 represents an empty name section (?right)
        // 0, 41 -> 41 is the type OPT
        // 16, 0 -> 4096 is the requestor's UDP payload size

        let sample = [112, 27, 1, 32, 0, 1, 0, 0, 0, 0, 0, 1, 3, 119, 119, 119, 6, 103, 111, 111, 103, 108, 101, 3, 99, 111, 109, 0, 0, 15, 0, 3, 0, 0, 41, 16, 0, 0, 0, 0, 0, 0, 0];
        let message = Message::from_bytes(&sample).unwrap();

        let mut bytes = Vec::new();
        println!("{:?}", message);
        message.write(&mut bytes).unwrap();
        println!("{:?}", bytes)
    }
}
