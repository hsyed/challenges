use std::{fmt, io};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::io::{Cursor, Read, Seek, Write};
use std::io::Result;

#[derive(Clone)]
pub struct Flags([u8; 2]);

impl Flags {
    fn from_bytes(left: u8, right: u8) -> Flags { Flags([left, right]) }

    fn write<W: MsgWrite>(&self, writer: &mut W) -> Result<()> { writer.write_all(&self.0) }

    /// QR, query/response flag. When 0, message is a query. When 1, message is response.
    pub fn qr(&self) -> u8 { self.0[0] >> 7 }

    pub fn set_qr(&mut self, qr: u8) {
        assert!(qr < 2, "qr must be 0 or 1");
        self.0[0] &= qr << 7;
    }

    /// Opcode, operation code. Tells receiving machine the intent of the message. Generally 0
    /// meaning normal query, However, there are other valid options such as 1 for reverse query and
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
#[derive(Debug, Clone)]
pub struct Header {
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

    fn write<W: MsgWrite>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.id.to_be_bytes())?;
        self.flags.write(writer)?;
        writer.write_all(&self.qdcount.to_be_bytes())?;
        writer.write_all(&self.ancount.to_be_bytes())?;
        writer.write_all(&self.nscount.to_be_bytes())?;
        writer.write_all(&self.arcount.to_be_bytes())?;
        Ok(())
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct Question {
    pub qname: String,
    pub qtype: u16,
    pub qclass: u16,
}

impl Question {
    fn read<R: Read + Seek>(r: &mut R) -> Result<Question> {
        Ok(
            Question {
                qname: read_labels_to_str(r)?,
                qtype: read_u16(r)?,
                qclass: read_u16(r)?,
            }
        )
    }

    fn write<W: MsgWrite>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_name(&self.qname)?;
        writer.write_all(&self.qtype.to_be_bytes())?;
        writer.write_all(&self.qclass.to_be_bytes())?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ResourceRecord {
    pub name: String,
    pub rtype: u16,
    pub rclass: u16,
    pub ttl: u32,
    pub rdlength: u16,
    pub rdata: Vec<u8>,
}

impl ResourceRecord {
    fn read<R: Read + Seek>(r: &mut R) -> Result<ResourceRecord> {
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

    fn read_all<R: Read + Seek>(r: &mut R, count: u16) -> Result<Vec<ResourceRecord>> {
        let mut records = Vec::with_capacity(count as usize);
        for _ in 0..count {
            records.push(ResourceRecord::read(r)?);
        }
        Ok(records)
    }

    fn write<W: MsgWrite>(&self, writer: &mut W) -> Result<()> {
        writer.write_name(&self.name)?;
        writer.write_all(&self.rtype.to_be_bytes())?;
        writer.write_all(&self.rclass.to_be_bytes())?;
        writer.write_all(&self.ttl.to_be_bytes())?;
        writer.write_all(&self.rdlength.to_be_bytes())?;
        writer.write_all(&self.rdata)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<ResourceRecord>,
    pub authorities: Vec<ResourceRecord>,
    pub additionals: Vec<ResourceRecord>,
}

impl Message {
    pub fn from_bytes(b: &[u8]) -> Result<Box<Message>> {
        let header = Header::from_bytes(b);
        let mut cur = Cursor::new(b);
        cur.set_position(12);

        let questions = match header.qdcount {
            0 => Vec::new(),
            1 => vec!(Question::read(&mut cur)
                .expect("could not parse question")),
            _ => return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported number of questions")
            ),
        };


        let answers = ResourceRecord::read_all(&mut cur, header.ancount)?;
        let authorities = ResourceRecord::read_all(&mut cur, header.nscount)?;
        let additionals = ResourceRecord::read_all(&mut cur, header.arcount)?;

        Ok(
            Box::new(Message {
                header,
                questions,
                answers,
                authorities,
                additionals,
            })
        )
    }

    pub fn to_udp_packet(&self) -> Result<Vec<u8>> {
        let mut writer = MessageWriter::new(Vec::new());
        self.write(&mut writer)?;
        Ok(writer.underlying)
    }

    fn write<W: MsgWrite>(&self, writer: &mut W) -> Result<()> {
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
trait MsgWrite {
    fn write_name(&mut self, name: &str) -> Result<()>;
    fn write_all(&mut self, buf: &[u8]) -> Result<()>;
}

#[derive(Debug)]
struct MessageWriter<W: Write> {
    underlying: W,
    label_tally: HashMap<String, u16>,
    pos: u16,
}

impl<W: Write> MessageWriter<W> {
    fn new(underlying: W) -> MessageWriter<W> {
        MessageWriter {
            underlying,
            label_tally: HashMap::new(),
            pos: 0,
        }
    }
}
impl<W: Write> MsgWrite for MessageWriter<W> {
    fn write_name(&mut self, name: &str) -> Result<()> {
        if name.is_empty() {
            self.write_all(&[0])
        } else {
            match self.label_tally.get(name) {
                Some(pos) => { // write pointer and terminate
                    self.write_all(&(pos | 0xC000).to_be_bytes())
                }
                None => {
                    self.label_tally.insert(name.to_string(), self.pos);

                    match name.split_once(".") {
                        None => {
                            self.write_all(&[name.len() as u8])?;
                            self.write_all(name.as_bytes())?;
                            self.write_all(&[0])
                        }
                        Some((left, rest)) => {
                            self.write_all(&[left.len() as u8])?;
                            self.write_all(left.as_bytes())?;
                            // TODO lets get rid of the recursion -- rust does not support tailrec.
                            self.write_name(rest)
                        }
                    }
                }
            }
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.underlying.write_all(&buf)?;
        self.pos += buf.len() as u16;
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

fn read_labels_to_str<R: Read + Seek>(r: &mut R) -> Result<String> {
    let mut qname = String::new();
    loop {
        match LabelKind::read(r)? {
            LabelKind::Absent => break,
            LabelKind::Data(len) => {
                if !qname.is_empty() {
                    qname.push('.');
                }
                let mut label = vec![0; len];
                r.read_exact(&mut label)?;
                qname.push_str(
                    std::str::from_utf8(&label).expect("invalid utf8 label")
                );
            }
            LabelKind::Pointer(offset) => {
                let pos = r.seek(io::SeekFrom::Current(0))?;
                r.seek(io::SeekFrom::Start(offset as u64))?;
                // TODO 1: is the as_str bad ? Can I switch the String used in qname to &str ?
                // TODO 2: is the recursion here bad ?
                qname.push_str(read_labels_to_str(r)?.as_str());
                r.seek(io::SeekFrom::Start(pos))?;
                break;
            }
        }
    }
    Ok(qname)
}

#[derive(Debug)]
#[derive(PartialEq)]
enum LabelKind {
    Absent,
    Data(usize),
    Pointer(u16),
}

impl LabelKind {
    fn read<R: Read>(r: &mut R) -> Result<LabelKind> {
        let mut b0 = [0u8; 1];
        r.read_exact(&mut b0)?;
        if b0[0] == 0 {
            Ok(LabelKind::Absent)
        } else if b0[0] & 0xC0 == 0xC0 {
            let mut b1 = [0u8; 1];
            r.read_exact(&mut b1)?;
            Ok(LabelKind::Pointer(u16::from_be_bytes([b0[0] & 0x3F, b1[0]])))
        } else {
            Ok(LabelKind::Data(b0[0] as usize))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_query_roundtrip() {
        let sample = [112, 27, 1, 32, 0, 1, 0, 0, 0, 0, 0, 1, 3, 119, 119, 119, 6, 103, 111, 111, 103, 108, 101, 3, 99, 111, 109, 0, 0, 15, 0, 3, 0, 0, 41, 16, 0, 0, 0, 0, 0, 0, 0];
        let message = Message::from_bytes(&sample).unwrap();
        assert_eq!(sample, message.to_udp_packet().unwrap().as_slice());
    }

    #[test]
    fn message_google_response_roundtrip() {
        let sample = [15, 245, 129, 128, 0, 1, 0, 1, 0, 0, 0, 1, 3, 119, 119, 119, 6, 103, 111, 111, 103, 108, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1, 192, 12, 0, 1, 0, 1, 0, 0, 0, 18, 0, 4, 142, 250, 179, 228, 0, 0, 41, 2, 0, 0, 0, 0, 0, 0, 0];
        let message = Message::from_bytes(&sample).unwrap();
        assert_eq!(sample, message.to_udp_packet().unwrap().as_slice());
    }

    #[test]
    fn labelkind_parsing() -> Result<()> {
        assert_eq!(LabelKind::read(&mut Cursor::new(&[0]))?, LabelKind::Absent);
        assert_eq!(LabelKind::read(&mut Cursor::new(&[1]))?, LabelKind::Data(1));
        assert_eq!(LabelKind::read(&mut Cursor::new(&[0xC0u8, 0x0C]))?, LabelKind::Pointer(12));
        Ok(())
    }

    #[test]
    fn message_tracker() {
        let mut tracker = MessageWriter::new(Vec::new());
        tracker.write_name("www.google.com").unwrap();
        tracker.write_name("google.com").unwrap();
        println!("{:?}", tracker.underlying); // TODO asserts
    }
}
