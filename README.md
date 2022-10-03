log4rs Test Utils
========================

This crate aims to solve the headache that often occurs when combining              
testing with logging. It is split into two halves depending on your use             
case: do you want to test your logs or log your tests?

# Testing your logs
If you want to test your logs, i.e. write tests that make assertions about          
your log output, then look to `log_testing`. Contents include:
* `MockAppender`, an appender that saves all logs to a `Vec<String>` for
  programmatic inspection.
* `logging_test_setup`, which handles test setup by configuring the logger and
  serializing all logging tests to make sure they don't conflict.
* `logging_test_setup_mock` which does the same, but automatically creates a
  `MockAppender` for you to save even more effort.

# Logging your tests
If you want to log your tests, i.e. set up a logger and capture the results         
for debugging your unit and integration tests, then look to `test_logging`.
Contents include:
* `TestConsoleAppender`, an appender which ensures logs are actually captured
  by the default test harness rather than spewed all over your lovely console.

## Example usage
```rust
use log::{error, info};
use log4rs_test_utils::logging_test_setup_mock;

#[test]
fn simple_mock_example() {
    let (_guard, logs_handle) = logging_test_setup_mock(None, None);

    info!("Hello, world!");
    error!("Oh, no!");
    info!("Goodbye, world.");

    let logs = logs_handle.lock().unwrap();
    assert_eq!(logs.len(), 3);
    assert_eq!(logs.iter().filter(|s| s.contains("INFO")).count(), 2);
    assert_eq!(logs.iter().filter(|s| s.contains("ERROR")).count(), 1);
    assert_eq!(logs.iter().filter(|s| s.contains(", world")).count(), 2);
}
```

## License
Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
