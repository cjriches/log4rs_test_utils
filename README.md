log4rs Test Utils
========================

This crate provides utilities that make testing logging with `log4rs` much easier.
Firstly, this crate provides the `MockAppender`: an `Appender` which logs all messages to a `Vec<String>` that can be inspected by test code.
Secondly, this crate provides the `logging_test_setup` and `logging_test_setup_mock` functions, which handle configuration and serialization of logging tests (unfortunately, logging tests cannot run in parallel due to the global logger configuration enforced by the `log` crate).

## Example usage
```rust
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
