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

### Example usage
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

# Logging your tests
If you want to log your tests, i.e. set up a logger and capture the results
for debugging your unit and integration tests, then look to `test_logging`.
Contents include:
* `TestConsoleAppender`, an appender which ensures logs are actually captured
  by the default test harness rather than spewed all over your lovely console.
* `init_logging_once`, which ensures your logging only gets initialized once,
  even if many tests are running  in parallel.
* `init_logging_once_for`, which does the same, but automatically creates a
  sensible config for the given targets.

### Example usage
```rust
use log::{info, LevelFilter};
use log4rs_test_utils::test_logging::init_logging_once_for;

fn some_fn(x: u32) -> u32 {
    info!(target: "foo", "some_fn called with {x}");
    panic!("This test will fail, but we'll have the logs to debug it.");
    x + 1
}

#[test]
fn some_test() {
    init_logging_once_for(vec!["foo", "bar"], LevelFilter::Debug, None);

    let y = some_fn(5);
    assert_eq!(y, 6);
}
```

## License
Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
