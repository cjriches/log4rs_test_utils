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
use std::sync::Once;

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

    /// Construct a sensible [`Config`] using a [`TestConsoleAppender`] and [`PatternEncoder`].
    ///
    /// If `targets` is empty, the root logger will be enabled at the given `level`.
    /// If `targets` is non-empty, the root logger will be disabled and all the
    /// given `targets` enabled at the given `level`.
    ///
    /// Defaults:
    /// * `level = LevelFilter::Trace`
    /// * `pattern = "{l} {M}::{L} {m}{n}"`
    ///
    /// Duplicate entries in `targets` are not allowed and will panic.
    pub fn make_config<'a, 'b>(
        targets: impl IntoIterator<Item = &'a str>,
        level: impl Into<Option<LevelFilter>>,
        pattern: impl Into<Option<&'b str>>,
    ) -> Config {
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
        if loggers.is_empty() {
            let root = Root::builder().appender(APPENDER_NAME).build(level);
            Config::builder().appender(appender).build(root).unwrap()
        } else {
            let root = Root::builder().build(LevelFilter::Off);
            Config::builder()
                .appender(appender)
                .loggers(loggers)
                .build(root)
                .unwrap()
        }
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

/// Synchronisation for [`init_logging_once`].
static INIT: Once = Once::new();

/// The first time this is called, the global logger will be initialized
/// with the given config. Once this returns, it is guaranteed that the global
/// logger has been configured exactly once, though not necessarily by this
/// invocation.
pub fn init_logging_once(config: Config) {
    INIT.call_once(|| {
        let result = log4rs::init_config(config);
        if result.is_err() {
            warn!(
                "init_logging_once tried to set the logger, but it had \
already been set by someone else!"
            );
        }
    });
}

/// A convenient wrapper for [`TestConsoleAppender::make_config`] and [`init_logging_once`],
/// which initializes logging once with a sensible config for the given targets.
///
/// See [`TestConsoleAppender::make_config`] for details.
pub fn init_logging_once_for<'a, 'b>(
    targets: impl IntoIterator<Item = &'a str>,
    level: impl Into<Option<LevelFilter>>,
    pattern: impl Into<Option<&'b str>>,
) {
    // No need to bother even constructing the config if we know init is already done.
    if INIT.is_completed() {
        return;
    }
    let config = TestConsoleAppender::make_config(targets, level, pattern);
    init_logging_once(config);
}
