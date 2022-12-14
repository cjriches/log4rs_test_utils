# Changelog

## v0.2.3
* Added `TestConsoleAppender::make_config` to get the config used by `init_logging_once_for`
  without actually initializing the logger with it.

## v0.2.2
* Fixed a bug where tests using `init_logging_once` could begin before the logger was fully initialized.
* Made repeated invocations of `init_logging_once_for` more efficient.

## v0.2.1
* Removed unnecessary features from dependencies.
  This avoids unnecessarily transitively enabling them in dependent crates.

## v0.2.0
* **BREAKING CHANGE:** Moved existing code under `log_testing` submodule.
* Added new features for configuring logging in normal, non-logging-related tests,
  all under the new `test_logging` submodule:
   - `TestConsoleAppender`
   - `init_logging_once`
   - `init_logging_once_for`
* Significantly improved documentation.
* Added feature flags for each half of the crate.

## v0.1.0
* Initial release
