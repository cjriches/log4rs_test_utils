//! Utilities for test logging, i.e. capturing log output during tests for debugging.

use log::Record;
use log4rs::{
    append::Append,
    encode::Encode,
};
use std::io::{self, Write};

use crate::string_buffer::StringBuffer;

/// An appender that uses [`print!`] internally. This is less performant than
/// a normal [`ConsoleAppender`](log4rs::append::console::ConsoleAppender),
/// but ensures output gets captured by the standard test harness.
#[derive(Debug)]
pub struct TestConsoleAppender {
    encoder: Box<dyn Encode>,
}

impl TestConsoleAppender {
    /// Create a new `TestConsoleAppender` with the given encoder.
    pub fn new(encoder: Box<dyn Encode>) -> Self {
        Self { encoder }
    }
}

impl Append for TestConsoleAppender {
    fn append(&self, record: &Record) -> anyhow::Result<()> {
        let mut log_line = StringBuffer::new();
        self.encoder.encode(&mut log_line, record)?;
        print!("{}", log_line.0);
        io::stdout().flush()?;
        Ok(())
    }

    fn flush(&self) {
        // no-op, since every log is flushed by default
    }
}
