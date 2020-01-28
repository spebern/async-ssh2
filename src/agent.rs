use crate::{aio::Aio, Error};
use ssh2::{self, PublicKey};
use std::{
    convert::From,
    future::Future,
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// See [`Agent`](ssh2::Agent).
pub struct Agent {
    inner: ssh2::Agent,
    aio: Arc<Option<Aio>>,
}

impl Agent {
    pub(crate) fn new(agent: ssh2::Agent, aio: Arc<Option<Aio>>) -> Self {
        Self { inner: agent, aio }
    }

    /// See [`connect`](ssh2::Agent::connect).
    pub async fn connect(&mut self) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.connect() })
    }

    /// See [`disconnect`](ssh2::Agent::disconnect).
    pub async fn disconnect(&mut self) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.disconnect() })
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
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.userauth(username, identity) })
    }
}
