# Changelog

## Unreleased
* Fixed a bug where tests using `init_logging_once` could begin before the logger was fully initialized.

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
