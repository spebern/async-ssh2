# async-ssh2-rs (deprecated)

PLEASE USE https://crates.io/crates/async-ssh2-lite INSTEAD.

[![Build Status](https://travis-ci.com/spebern/async-ssh2.svg?branch=master)](https://travis-ci.com/spebern/async-ssh2)
[![Build Status](https://github.com/spebern/async-ssh2/workflows/linux/badge.svg)](https://github.com/spebern/async-ssh2/actions?workflow=linux)
[![Build Status](https://github.com/spebern/async-ssh2/workflows/Windows/badge.svg)](https://github.com/spebern/async-ssh2/actions?workflow=Windows)
[![Build Status](https://github.com/spebern/async-ssh2/workflows/macOS/badge.svg)](https://github.com/spebern/async-ssh2/actions?workflow=macOS)

[Documentation](https://docs.rs/async-ssh2)

Async wrapper over [ssh2-rs](https://github.com/alexcrichton/ssh2-rs).

## Usage

```toml
# Cargo.toml
[dependencies]
async-ssh2 = { version = "0.1", git = "https://github.com/spebern/async-ssh2.git" }
```

## Building on OSX 10.10+

This library depends on OpenSSL. To get OpenSSL working follow the
[`openssl` crate's instructions](https://github.com/sfackler/rust-openssl#macos).

You can enable the `vendored-openssl` feature
to have `libssh2` built against a statically built version of openssl as [described
here](https://docs.rs/openssl/0.10.24/openssl/#vendored).
