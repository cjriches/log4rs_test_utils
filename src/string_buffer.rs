use log4rs::encode;
use std::io;

/// A simple string buffer that can be written to by an encoder.
/// We assume UTF-8 encoding.
#[derive(Debug)]
pub struct StringBuffer(pub String);

impl StringBuffer {
    pub fn new() -> Self {
        Self(String::new())
    }
}

impl io::Write for StringBuffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let buf_encoded = String::from_utf8(buf.to_vec())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid UTF-8"))?;
        self.0.push_str(&buf_encoded);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl encode::Write for StringBuffer {}
