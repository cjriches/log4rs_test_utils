//! This crate aims to solve the headache that often occurs when combining
//! testing with logging. It is split into two halves depending on your use
//! case: do you want to log your tests or test your logs?
//!
//! # Logging your tests
//! If you want to log your tests, i.e. set up a logger and capture the results
//! for debugging your unit and integration tests, then look to
//! [`test_logging`]. Contents include:
//! * [`TestConsoleAppender`](test_logging::TestConsoleAppender), an appender
//!   which ensures logs are actually captured by the default test harness rather
//!   than spewed all over your lovely console.
//! * [`init_logging_once`](test_logging::init_logging_once), which ensures
//!   your logging only gets initialized once, even if many tests are running
//!   in parallel.
//! * [`init_logging_once_for`](test_logging::init_logging_once_for), which
//!   does the same, but automatically creates a sensible config for the given
//!   targets.
//!
//! # Testing your logs
//! If you want to test your logs, i.e. write tests that make assertions about
//! your log output, then look to [`log_testing`]. Contents include:
//! * [`MockAppender`](log_testing::MockAppender), an appender that saves all
//!   logs to a [`Vec<String>`] for programmatic inspection.
//! * [`logging_test_setup`](log_testing::logging_test_setup), which handles test
//!   setup by configuring the logger and serializing all logging tests to make
//!   sure they don't conflict.
//! * [`logging_test_setup_mock`](log_testing::logging_test_setup_mock) which does
//!   the same, but automatically creates a [`MockAppender`](log_testing::MockAppender)
//!   for you to save even more effort.
//!
//! # Features
//! The two halves of this module are feature-gated, so you can disable anything
//! you don't want. Both features are enabled by default.
//!
//! |    Feature     |           Enables           |
//! | -------------- | --------------------------- |
//! | `log_testing`  | the [`log_testing`] module  |
//! | `test_logging` | the [`test_logging`] module |

mod string_buffer;

/// Requires the `log_testing` feature.
#[cfg(feature = "log_testing")]
pub mod log_testing;

/// Requires the `test_logging` feature.
#[cfg(feature = "test_logging")]
pub mod test_logging;
