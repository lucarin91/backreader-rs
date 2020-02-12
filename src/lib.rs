use std::io::Read;
use std::io::Result;
use std::io::Seek;
use std::io::SeekFrom;
use std::mem;

const NEW_LINE: u8 = 10;
const BUF_SIZE: usize = 512;

pub trait BackBufRead {
    fn read_line(&mut self, buf: &mut String) -> Result<usize>;
    fn lines(self) -> BackLines<Self>
    where
        Self: Sized,
    {
        BackLines { buf: self }
    }
}

pub struct BackBufReader<R> {
    inner: R,
    block_buf: Vec<u8>,
    lines: Vec<String>,
    pos: u64,
    seek: i64,
    begin: bool,
    rest_buf: Vec<u8>,
    end: bool,
}

impl<R: Read + Seek> BackBufReader<R> {
    pub fn new(inner: R) -> BackBufReader<R> {
        BackBufReader {
            inner,
            block_buf: Vec::with_capacity(BUF_SIZE),
            lines: Vec::new(),
            pos: 0,
            seek: -(BUF_SIZE as i64),
            begin: true,
            rest_buf: Vec::new(),
            end: false,
        }
    }
}

impl<R: Read + Seek> BackBufRead for BackBufReader<R> {
    fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        if let Some(line) = self.lines.pop() {
            mem::replace(buf, line);
            return Ok(1usize);
        }

        if self.end {
            if !self.rest_buf.is_empty() {
                mem::replace(buf, String::from_utf8(self.rest_buf.clone()).unwrap());
                self.rest_buf.clear();
                return Ok(1usize);
            } else {
                return Ok(0);
            }
        }

        let seek_result = self.inner.seek(SeekFrom::End(self.seek));
        match seek_result {
            Ok(p) => {
                self.block_buf.resize(512, 0u8);
                self.inner
                    .read_exact(&mut self.block_buf)
                    .expect("error read exact");
                self.pos = p;
            }
            Err(_) => {
                self.inner.seek(SeekFrom::Start(0)).expect("seek to start");
                if self.begin {
                    self.inner
                        .read_to_end(&mut self.block_buf)
                        .expect("error read to end");
                } else {
                    self.block_buf.resize(self.pos as usize, 0u8);
                    self.inner
                        .read_exact(&mut self.block_buf)
                        .expect("error read exact final")
                }
                self.end = true;
            }
        }

        if !self.rest_buf.is_empty() {
            self.block_buf.append(&mut self.rest_buf);
        }
        self.rest_buf.clear();

        // let str_block = String::from_utf8(self.block_buf.clone()).unwrap();
        // println!("debug {}", str_block);

        let mut line_buf = Vec::new();
        let mut before_newline = true;
        for c in self.block_buf.iter() {
            // print!(" [{}] ", *c);
            if before_newline {
                if *c == NEW_LINE {
                    before_newline = false;
                }
                self.rest_buf.push(*c);
            } else if *c == NEW_LINE {
                let line = String::from_utf8(line_buf.clone()).unwrap();
                self.lines.push(line);
                line_buf.clear();
            } else {
                line_buf.push(*c);
            }
        }
        self.seek -= 512;
        self.begin = false;

        let line = self.lines.pop().unwrap();
        mem::replace(buf, line);
        Ok(1usize)
    }
}

pub struct BackLines<B> {
    buf: B,
}

impl<B: BackBufRead> Iterator for BackLines<B> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Result<String>> {
        let mut buf = String::new();
        match self.buf.read_line(&mut buf) {
            Ok(0) => None,
            Ok(_n) => {
                if buf.ends_with('\n') {
                    buf.pop();
                    if buf.ends_with('\r') {
                        buf.pop();
                    }
                }
                Some(Ok(buf))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;

    #[test]
    fn small_file() {
        let filepath = "res/sequence.txt";
        let f = File::open(filepath).expect("error open file");
        let lines: Vec<String> = BackBufReader::new(f)
            .lines()
            .filter_map(Result::ok)
            .collect();

        let f = File::open(filepath).expect("error open file");
        let mut lines2: Vec<String> = BufReader::new(f).lines().filter_map(Result::ok).collect();
        lines2.reverse();

        assert_eq!(lines, lines2);
    }
}
