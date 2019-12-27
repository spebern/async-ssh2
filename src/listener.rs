use crate::{channel::Channel, into_the_future, io::Io};
use mio::Ready;
use ssh2::{self, Error};
use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::io::PollEvented;

/// A listener represents a forwarding port from the remote server.
///
/// New channels can be accepted from a listener which represent connections on
/// the remote server's port.
pub struct Listener {
    inner: ssh2::Listener,
    poll_evented: Arc<Option<PollEvented<Io>>>,
}

impl Listener {
    pub(crate) fn new(
        listener: ssh2::Listener,
        poll_evented: Arc<Option<PollEvented<Io>>>,
    ) -> Self {
        Self {
            inner: listener,
            poll_evented,
        }
    }

    fn poll_evented(&self) -> Arc<Option<PollEvented<Io>>> {
        self.poll_evented.clone()
    }

    /// Accept a queued connection from this listener.
    pub async fn accept(&mut self) -> Result<Channel, Error> {
        let poll_evented = self.poll_evented();
        let channel = into_the_future!(poll_evented; &mut || { self.inner.accept() })?;
        Ok(Channel::new(channel, self.poll_evented()))
    }
}
