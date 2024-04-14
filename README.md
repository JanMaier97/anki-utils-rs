# anki-utils-rs

A personal Anki utility tool written in Rust, that uses the AnkiConnect plugin to provide its functionality.

# Usage requirements

Install the [AnkiConnect](https://ankiweb.net/shared/info/2055492159) plugin in Anki and restart. Anki has to be running for this utility tool to function.

# Utilities

## Kanji Extractor

**coming some time later **

## Note Validator

Todos:
- [ ] Add 'tag required' validation
- [ ] Add 'tag starts with' validation
- [ ] Add integration tests
- [ ] Add logging
- [ ] Add value sets validation
- [ ] CLI: Parameter for different port
- [ ] CLI: Parameter for different url (default: localhost)
- [ ] Refactor anki request api
- [x] CLI: Parameter for config
- [x] CLI: Parameter to filter error types
- [x] CLI: Parameter to filter field names
- [x] CLI: Parameter to open notes in the Anki browser
- [x] CLI: Print validation errors in a table format to the browser
- [x] Print error messages to stderr
- [x] Refactor table print and write to buffer instead
- [x] Refactor validation libary 
- [x] Try error handling with `anyhow` crate
- [x] Try error handling with `this-error` crate
- [x] Validate note model and field values provided in config

**Notes**:
- For the anki rest api wrapper these guides could help:
    - https://plume.benboeckel.net/~/JustAnotherBlog/designing-rust-bindings-for-rest-ap-is
    - https://dev.to/rogertorres/rest-api-wrapper-with-rust-mk4
    - https://nullderef.com/blog/web-api-client/
- Support different versions
- Support different http clients
- Support async and blocking requests
