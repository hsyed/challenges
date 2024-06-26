use std::io::Read;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "wc")]
struct Cli {
    #[arg(short = 'c')]
    count_bytes: bool,
    #[arg(short = 'l')]
    count_lines: bool,
    #[arg(short = 'w')]
    count_words: bool,
    #[arg(short = 'm')]
    count_chars: bool,
    file: Option<String>,
}

trait ByteVisitor {
    fn result(&self) -> String;
    fn visit(&mut self, byte: u8);
    fn done(&mut self);
}

struct ByteCounter {
    count: u64,
}

impl ByteVisitor for ByteCounter {
    fn result(&self) -> String { return self.count.to_string(); }
    fn visit(&mut self, _: u8) { self.count += 1; }
    fn done(&mut self) {}
}

struct LineCounter {
    count: u64,
}

impl ByteVisitor for LineCounter {
    fn result(&self) -> String { return self.count.to_string(); }
    fn visit(&mut self, byte: u8) { if byte == b'\n' { self.count += 1; } }
    fn done(&mut self) {}
}

struct WordCounter {
    count: u64,
    in_word: bool
}

impl ByteVisitor for WordCounter {
    fn result(&self) -> String { return self.count.to_string(); }
    fn visit(&mut self, byte: u8) {
        if byte == b' ' || byte == b'\n' || byte == b'\t' {
            if self.in_word {
                self.count += 1;
                self.in_word = false;
            }
        } else {
            self.in_word = true;
        }
    }
    fn done(&mut self) {
        if self.in_word {
            self.count += 1;
            self.in_word = false;
        }
    }
}

fn visit_source<R: Read>(source: R, visitors: &mut [&mut dyn ByteVisitor]) {
    for byte in source.bytes() {
        let byte = byte.unwrap();
        for visitor in visitors.iter_mut() {
            visitor.visit(byte);
        }
    }

    visitors.iter_mut().for_each(|v| v.done());
}

struct CharCounter {
    count: u64,
    tally: Vec<u8>
}

impl ByteVisitor for CharCounter {
    fn result(&self) -> String { return self.count.to_string(); }

    fn visit(&mut self, byte: u8) {
        self.tally.push(byte);
        // from_utf8 is a type conversion / allocation free.
        if let Ok(_) = std::str::from_utf8(&self.tally) {
            self.tally.clear();
            self.count += 1;
        }
    }

    fn done(&mut self) {
        if self.tally.len() > 0 {
            panic!("Invalid UTF-8");
        }
    }
}

// 🤣<-- this is not an ASCII rune (test data).
fn main() {
    let mut args = Cli::parse();

    if !(args.count_lines || args.count_words || args.count_bytes || args.count_chars) {
        args.count_lines = true;
        args.count_words = true;
        args.count_bytes = true;
    }

    let mut lc = LineCounter { count: 0 };
    let mut wc = WordCounter { count: 0, in_word: false };
    let mut bc = ByteCounter { count: 0 };
    let mut cc = CharCounter { count: 0, tally: Vec::new() };

    let mut visitors: Vec<&mut dyn ByteVisitor> = Vec::new();

    if args.count_lines { visitors.push(&mut lc); }
    if args.count_words { visitors.push(&mut wc); }
    if args.count_bytes { visitors.push(&mut bc); }
    if args.count_chars { visitors.push(&mut cc); }

    if let Some(ref filen) = args.file {
        let file = std::fs::File::open(&filen).unwrap();
        visit_source(file, visitors.as_mut_slice());
    } else {
        visit_source(std::io::stdin(), visitors.as_mut_slice());
    }

    let results = visitors.iter().map(|v| v.result()).fold(String::new(), |acc, s| {
        if acc.is_empty() {
            s
        } else {
            acc + " " + &s
        }
    });

    println!("{} {}", results, args.file.unwrap_or("".to_string()));
}