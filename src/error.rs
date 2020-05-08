use std::{convert::From, error, fmt, io};

/// Representation of an error.
#[derive(Debug)]
pub enum Error {
    // An error that can occur within libssh2.
    SSH2(ssh2::Error),
    // An io error.
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => e.fmt(f),
            Error::SSH2(e) => e.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::SSH2(e) => e.message(),
            Error::Io(e) => e.description(),
        }
    }
}

impl From<ssh2::Error> for Error {
    fn from(e: ssh2::Error) -> Error {
        Error::SSH2(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}
