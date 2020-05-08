use crate::{util::run_ssh2_fn, Error};
use smol::Async;
use ssh2::{self, PublicKey};
use std::{convert::From, net::TcpStream, sync::Arc};

/// See [`Agent`](ssh2::Agent).
pub struct Agent {
    inner: ssh2::Agent,
    stream: Arc<Async<TcpStream>>,
}

impl Agent {
    pub(crate) fn new(agent: ssh2::Agent, stream: Arc<Async<TcpStream>>) -> Self {
        Self {
            inner: agent,
            stream,
        }
    }

    /// See [`connect`](ssh2::Agent::connect).
    pub async fn connect(&mut self) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), || self.inner.connect()).await
    }

    /// See [`disconnect`](ssh2::Agent::disconnect).
    pub async fn disconnect(&mut self) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), || self.inner.disconnect()).await
    }

    /// See [`list_identities`](ssh2::Agent::list_identities).
    pub fn list_identities(&mut self) -> Result<(), Error> {
        self.inner.list_identities().map_err(From::from)
    }

    /// See [`identities`](ssh2::Agent::identities).
    pub fn identities(&self) -> Result<Vec<PublicKey>, Error> {
        self.inner.identities().map_err(From::from)
    }

    /// See [`userauth`](ssh2::Agent::userauth).
    pub async fn userauth(&self, username: &str, identity: &PublicKey) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), || {
            self.inner.userauth(username, identity)
        })
        .await
    }
}
