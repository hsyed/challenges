use std::{fmt, io};
use std::fmt::Formatter;
use std::io::{Cursor, Read, Write};
use std::io::Result;

pub struct Flags([u8; 2]);

impl Flags {
    fn from_bytes(left: u8, right: u8) -> Flags { Flags([left, right]) }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> { writer.write_all(&self.0) }

    /// QR, query/response flag. When 0, message is a query. When 1, message is response.
    pub fn qr(&self) -> u8 { self.0[0] >> 7 }
    /// Opcode, operation code. Tells receiving machine the intent of the message. Generally 0
    /// meaning normal query, However there are other valid options such as 1 for reverse query and
    /// 2 for server status.
    pub fn opcode(&self) -> u8 { (self.0[0] >> 3) & 0x0F }
    /// AA, authoritative answer. Set only when the responding machine is the authoritative name
    /// server of the queried domain.
    pub fn aa(&self) -> u8 { (self.0[0] >> 2) & 0x01 }
    /// TC, truncated. Set if packet is larger than the UDP maximum size of 512 bytes.
    pub fn tc(&self) -> u8 { (self.0[0] >> 1) & 0x01 }
    /// RD, recursion desired. If 0, the query is an iterative query. If 1, the query is recursive.
    pub fn rd(&self) -> u8 { self.0[0] & 0x01 }
    /// RA, recursion available. Set on response if the server supports recursion.
    pub fn ra(&self) -> u8 { self.0[1] >> 7 }
    /// Z. Reserved for future use, must be set to 0 on all queries and responses.
    pub fn z(&self) -> u8 { (self.0[1] >> 6) & 0x01 }
    /// AD, authentic data. Used in DNSSEC. Considered part of Z in older machines.
    pub fn ad(&self) -> u8 { (self.0[1] >> 5) & 0x01 }
    /// CD, checking disabled. Used in DNSSEC. Considered part of Z in older machines.
    pub fn cd(&self) -> u8 { (self.0[1] >> 4) & 0x01 }
    /// Rcode, return code. It will generally be 0 for no error, or 3 if the name does not exist.
    pub fn rcode(&self) -> u8 { self.0[1] & 0x0F }
}

impl fmt::Debug for Flags {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Flags")
            .field("qr", &self.qr())
            .field("opcode", &self.opcode())
            .field("aa", &self.aa())
            .field("tc", &self.tc())
            .field("rd", &self.rd())
            .field("ra", &self.ra())
            .field("z", &self.z())
            .field("ad", &self.ad())
            .field("cd", &self.cd())
            .field("rcode", &self.rcode())
            .finish()
    }
}

// The RFC 1035 is a bit outdated with regards to the flags.
// This resource seems to be comprehensive: https://www.catchpoint.com/blog/how-dns-works
// Also check Wireshark.
#[derive(Debug)]
struct Header {
    /// ID, a 16-bit identifier assigned by the program that generates any kind of query.
    pub id: u16,
    pub flags: Flags,
    /// QDCount, an unsigned 16-bit integer specifying the number of entries in the question section.
    pub qdcount: u16,
    /// ANCount, an unsigned 16-bit integer specifying the number of resource records in the answer.
    pub ancount: u16,
    /// NSCount, an unsigned 16-bit integer specifying the number of name server resource records in
    /// the response.
    pub nscount: u16,
    /// ArCount, an unsigned 16-bit integer specifying the number of resource records in the
    pub arcount: u16,
}

impl Header {
    fn from_bytes(b: &[u8]) -> Header {
        Header {
            id: u16::from_be_bytes([b[0], b[1]]),
            flags: Flags::from_bytes(b[2], b[3]),
            qdcount: u16::from_be_bytes([b[4], b[5]]),
            ancount: u16::from_be_bytes([b[6], b[7]]),
            nscount: u16::from_be_bytes([b[8], b[9]]),
            arcount: u16::from_be_bytes([b[10], b[11]]),
        }
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.id.to_be_bytes())?;
        self.flags.write(writer)?;
        writer.write_all(&self.qdcount.to_be_bytes())?;
        writer.write_all(&self.ancount.to_be_bytes())?;
        writer.write_all(&self.nscount.to_be_bytes())?;
        writer.write_all(&self.arcount.to_be_bytes())?;
        Ok(())
    }
}

#[derive(Debug)]
struct Question {
    pub qname: String,
    pub qtype: u16,
    pub qclass: u16,
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
        // do not process empty string as the split will return a single empty string
        if !self.name.is_empty() {
            let name = self.name.split('.');
            for label in name {
                writer.write_all(&[label.len() as u8])?;
                writer.write_all(label.as_bytes())?;
            }
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

        println!("{:?}", message);

        let mut bytes = Vec::new();
        message.write(&mut bytes).unwrap();

        assert_eq!(sample, bytes.as_slice());
    }
}
