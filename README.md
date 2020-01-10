# async-ssh2-rs

[Documentation](https://docs.rs/async-ssh2)

Async wrapper over [ssh2-rs](https://github.com/alexcrichton/ssh2-rs).

## Usage

```toml
# Cargo.toml
[dependencies]
async-ssh2 = "todo"
```

## Building on OSX 10.10+

This library depends on OpenSSL. To get OpenSSL working follow the
[`openssl` crate's instructions](https://github.com/sfackler/rust-openssl#macos).

You can enable the `vendored-openssl` feature
to have `libssh2` built against a statically built version of openssl as [described
here](https://docs.rs/openssl/0.10.24/openssl/#vendored).
