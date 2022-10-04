//! Utilities for test logging, i.e. capturing log output during tests for debugging.
//!
//! ```rust
//! use log::{info, LevelFilter};
//! use log4rs_test_utils::test_logging::init_logging_once_for;
//!
//! fn some_fn(x: u32) -> u32 {
//!     info!(target: "foo", "some_fn called with {x}");
//!     panic!("This test will fail, but we'll have the logs to debug it.");
//!     x + 1
//! }
//!
//! #[test]
//! fn some_test() {
//!     init_logging_once_for(vec!["foo", "bar"], LevelFilter::Debug, None);
//!
//!     let y = some_fn(5);
//!     assert_eq!(y, 6);
//! }
//! ```

use log::{warn, LevelFilter, Record};
use log4rs::{
    append::Append,
    config::{Appender, Config, Logger, Root},
    encode::{pattern::PatternEncoder, Encode},
};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::string_buffer::StringBuffer;

/// An appender that uses [`print!`] internally. This is less performant than
/// a normal `ConsoleAppender`, but ensures output gets captured by the
/// standard test harness.
#[derive(Debug)]
pub struct TestConsoleAppender {
    encoder: Box<dyn Encode>,
}

impl TestConsoleAppender {
    /// Create a new [`TestConsoleAppender`] with the given encoder.
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

/// The first time this is called, the global logger will be initialized
/// with the given config. Subsequent calls are no-ops, so this is safe
/// to call from every test.
pub fn init_logging_once(config: Config) {
    static INIT_DONE: AtomicBool = AtomicBool::new(false);

    if !INIT_DONE.swap(true, Ordering::Relaxed) {
        let result = log4rs::init_config(config);
        if result.is_err() {
            warn!(
                "init_logging_once tried to set the logger, but it had \
already been set by someone else!"
            );
        }
    }
}

/// Like [`init_logging_once`], but constructs a sensible config for the given targets.
///
/// If `targets` is empty, the root logger will be configured at the given `level`.
/// If `targets` is non-empty, the root logger will be disabled and all the
/// given `targets` configured at the given `level`.
/// The configured appender will be a [`TestConsoleAppender`] with a [`PatternEncoder`].
///
/// Defaults:
/// * `level = LevelFilter::Trace`
/// * `pattern = {l} {M}::{L} {m}{n}`
///
/// Duplicate entries in `targets` are not allowed and will panic.
pub fn init_logging_once_for<'a, 'b>(
    targets: impl IntoIterator<Item = &'a str>,
    level: impl Into<Option<LevelFilter>>,
    pattern: impl Into<Option<&'b str>>,
) {
    let level = level.into().unwrap_or(LevelFilter::Trace);
    let pattern = pattern.into().unwrap_or("{l} {M}::{L} {m}{n}");
    const APPENDER_NAME: &str = "appender";

    // Create encoder and appender.
    let encoder = Box::new(PatternEncoder::new(pattern));
    let console = Box::new(TestConsoleAppender::new(encoder));
    let appender = Appender::builder().build(APPENDER_NAME, console);

    // Create a logger for each target.
    let mut loggers = Vec::new();
    for target in targets {
        let logger = Logger::builder()
            .appender(APPENDER_NAME)
            .build(target, level);
        loggers.push(logger);
    }

    // Create the root logger and final config.
    let config = if loggers.is_empty() {
        let root = Root::builder().appender(APPENDER_NAME).build(level);
        Config::builder().appender(appender).build(root).unwrap()
    } else {
        let root = Root::builder().build(LevelFilter::Off);
        Config::builder()
            .appender(appender)
            .loggers(loggers)
            .build(root)
            .unwrap()
    };

    // Pass the config to the initializer.
    init_logging_once(config);
}
