//! Utilities for log testing, i.e. tests which ensure the log output is correct.
//!
//! These are most useful to developers of [`log4rs`] extension libraries, although
//! any application could choose to test its log output.
//! Unless you fall into that narrow camp, you probably want [`test_logging`].
//!
//! ```rust
//! use log::{error, info};
//! use log4rs_test_utils::log_testing::logging_test_setup_mock;
//!
//! #[test]
//! fn simple_mock_example() {
//!     let (_guard, logs_handle) = logging_test_setup_mock(None, None);
//!
//!     info!("Hello, world!");
//!     error!("Oh, no!");
//!     info!("Goodbye, world.");
//!
//!     let logs = logs_handle.lock().unwrap();
//!     assert_eq!(logs.len(), 3);
//!     assert_eq!(logs.iter().filter(|s| s.contains("INFO")).count(), 2);
//!     assert_eq!(logs.iter().filter(|s| s.contains("ERROR")).count(), 1);
//!     assert_eq!(logs.iter().filter(|s| s.contains(", world")).count(), 2);
//! }
//! ```

use lazy_static::lazy_static;
use log::{LevelFilter, Record};
use log4rs::{
    append::Append,
    config::{Appender, Root},
    encode::{pattern::PatternEncoder, Encode},
    Config, Handle,
};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::string_buffer::StringBuffer;

/// A thread-safe handle to the list of log messages written by a [`MockAppender`].
pub type LogsHandle = Arc<Mutex<Vec<String>>>;

/// A mock appender that encodes its messages to a [`Vec<String>`].
#[derive(Debug)]
pub struct MockAppender {
    logs: LogsHandle,
    encoder: Box<dyn Encode>,
}

impl MockAppender {
    /// Create a new [`MockAppender`], returning it along with a handle to its
    /// log buffer.
    pub fn new(encoder: Box<dyn Encode>) -> (Self, LogsHandle) {
        let logs: LogsHandle = Default::default();
        let appender = Self {
            logs: logs.clone(),
            encoder,
        };
        (appender, logs)
    }
}

impl Append for MockAppender {
    fn append(&self, record: &Record) -> anyhow::Result<()> {
        let mut log_line = StringBuffer::new();
        self.encoder.encode(&mut log_line, record).unwrap();
        self.logs.lock().unwrap().push(log_line.0);
        Ok(())
    }

    fn flush(&self) {
        // no-op
    }
}

lazy_static! {
    /// A handle to the global logger that will be created on first access.
    /// Can be used to set and re-set the config.
    static ref HANDLE: Handle = {
        let root = Root::builder().build(LevelFilter::Off);
        let config = Config::builder().build(root).unwrap();
        log4rs::init_config(config).unwrap()
    };
}

/// A mutex for ensuring tests execute sequentially.
/// Unfortunately there is no safe way to parallelize logging tests thanks to
/// the global logger and the fact that the target is chosen by the code doing
/// the logging.
static TEST_MUTEX: Mutex<()> = Mutex::new(());

/// Call this at the start of a logging test to configure the logger.
///
/// The returned mutex guard ensures no other logging test can execute
/// simultaneously; this is vital for correctness since there is only one
/// global logger. Do not drop it until the end of the test.
pub fn logging_test_setup(config: Config) -> MutexGuard<'static, ()> {
    let guard = TEST_MUTEX.lock();
    HANDLE.set_config(config);
    // Since the mutex only holds a `()`, poison is irrelevant and we can ignore it.
    // In fact, we *should* ignore it as the mutex will definitely become poisoned
    // if any test fails - we don't want this to panic other tests.
    match guard {
        Ok(guard) => guard,
        Err(poison) => poison.into_inner(),
    }
}

/// A convenient wrapper for [`logging_test_setup`] that configures the global
/// logger with a fresh [`MockAppender`].
///
/// Defaults:
/// * `level = LevelFilter::Trace`
/// * `encoder = PatternEncoder with pattern "{l} {t} {m}"`
pub fn logging_test_setup_mock(
    level: impl Into<Option<LevelFilter>>,
    encoder: impl Into<Option<Box<dyn Encode>>>,
) -> (MutexGuard<'static, ()>, LogsHandle) {
    const APPENDER_NAME: &str = "mock";
    let encoder = encoder
        .into()
        .unwrap_or_else(|| Box::new(PatternEncoder::new("{l} {t} {m}")));
    let (mock, logs) = MockAppender::new(encoder);
    let appender = Appender::builder().build(APPENDER_NAME, Box::new(mock));
    let level = level.into().unwrap_or(LevelFilter::Trace);
    let root = Root::builder().appender(APPENDER_NAME).build(level);
    let config = Config::builder().appender(appender).build(root).unwrap();
    (logging_test_setup(config), logs)
}

#[cfg(test)]
mod tests {
    use super::*;

    use log::{error, info, warn};

    #[test]
    fn simple_mock_example() {
        let (_guard, logs_handle) = logging_test_setup_mock(None, None);

        info!("Hello, world!");
        error!("Oh, no!");
        info!("Goodbye, world.");

        let logs = logs_handle.lock().unwrap();
        assert_eq!(logs.len(), 3);
        for line in logs.iter() {
            assert!(line.contains("log4rs_test_utils::log_testing::tests"));
        }
        assert_eq!(logs.iter().filter(|s| s.contains("INFO")).count(), 2);
        assert_eq!(logs.iter().filter(|s| s.contains("ERROR")).count(), 1);
        assert_eq!(logs.iter().filter(|s| s.contains(", world")).count(), 2);
    }

    #[test]
    fn custom_mock_example() {
        let encoder: Box<dyn Encode> = Box::new(PatternEncoder::new("{m}"));
        let (_guard, logs_handle) = logging_test_setup_mock(LevelFilter::Warn, encoder);

        info!("this will not appear");
        warn!("this will appear");
        error!("so will this");

        let logs = logs_handle.lock().unwrap();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0], "this will appear");
    }
}
