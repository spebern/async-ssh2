use crate::{into_the_future, io::Io};
use mio::Ready;
use ssh2::{self, Error, Identities, PublicKey};
use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::io::PollEvented;

pub struct Agent {
    inner: ssh2::Agent,
    poll_evented: Arc<Option<PollEvented<Io>>>,
}

impl Agent {
    pub(crate) fn new(agent: ssh2::Agent, poll_evented: Arc<Option<PollEvented<Io>>>) -> Self {
        Self {
            inner: agent,
            poll_evented,
        }
    }

    fn poll_evented(&self) -> Arc<Option<PollEvented<Io>>> {
        self.poll_evented.clone()
    }

    /// Connect to an ssh-agent running on the system.
    pub async fn connect(&mut self) -> Result<(), Error> {
        let poll_evented = self.poll_evented();
        into_the_future!(poll_evented; &mut || { self.inner.connect() })
    }

    /// Close a connection to an ssh-agent.
    pub async fn disconnect(&mut self) -> Result<(), Error> {
        let poll_evented = self.poll_evented();
        into_the_future!(poll_evented; &mut || { self.inner.disconnect() })
    }

    /// Request an ssh-agent to list of public keys, and stores them in the
    /// internal collection of the handle.
    ///
    /// Call `identities` to get the public keys.
    pub fn list_identities(&mut self) -> Result<(), Error> {
        self.inner.list_identities()
    }

    /// Get an iterator over the identities of this agent.
    pub fn identities(&self) -> Identities {
        self.inner.identities()
    }

    /// Attempt public key authentication with the help of ssh-agent.
    pub async fn userauth(&self, username: &str, identity: &PublicKey<'_>) -> Result<(), Error> {
        let poll_evented = self.poll_evented();
        into_the_future!(poll_evented; &mut || { self.inner.userauth(username, identity) })
    }
}
