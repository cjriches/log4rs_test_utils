use lazy_static::lazy_static;
use log::{LevelFilter, Record};
use log4rs::{
    append::Append,
    config::{Appender, Root},
    encode::{self, pattern::PatternEncoder, Encode},
    Config, Handle,
};
use std::io::{self, Write};
use std::sync::{Arc, Mutex, MutexGuard};

/// A thread-safe handle to a list of log messages.
pub type LogsHandle = Arc<Mutex<Vec<String>>>;

/// A mock appender useful for testing, which simply encodes its
/// messages and adds them to a `Vec<String>`.
#[derive(Debug)]
pub struct MockAppender {
    logs: LogsHandle,
    encoder: Box<dyn Encode>,
}

impl MockAppender {
    /// Create a new `MockAppender`, returning it along with a handle to its log buffer.
    pub fn new(encoder: impl Into<Option<Box<dyn Encode>>>) -> (Self, LogsHandle) {
        let encoder = encoder
            .into()
            .unwrap_or_else(|| Box::new(PatternEncoder::default()));
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

/// An appender that uses `print!()` internally. This is less performant than
/// a normal `ConsoleAppender`, but ensures output gets captured by the
/// standard test harness.
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

/// A simple string buffer that can be written to by an encoder.
/// We assume UTF-8 encoding.
#[derive(Debug)]
struct StringBuffer(pub String);

impl StringBuffer {
    pub fn new() -> Self {
        Self(String::new())
    }
}

impl Write for StringBuffer {
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
/// the global logger and the fact that the target is chosen by the code doing the logging.
static TEST_MUTEX: Mutex<()> = Mutex::new(());

/// Call this at the start of a test to configure the logger. The returned mutex guard ensures no
/// other tests can execute simultaneously; this is vital for correctness. Do not drop it until
/// the end of the test.
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

/// A convenient wrapper for `logging_test_setup` that configures the global logger with
/// a fresh `MockAppender`.
/// If not supplied, the level defaults to `Trace`, and the encoder to a `PatternEncoder` with
/// the pattern `{l} {t} {m}`.
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
            assert!(line.contains("log4rs_test_utils::tests"));
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
