use crate::{aio::Aio, channel::Channel, into_the_future};

use ssh2::{self, Error};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// A listener represents a forwarding port from the remote server.
///
/// New channels can be accepted from a listener which represent connections on
/// the remote server's port.
pub struct Listener {
    inner: ssh2::Listener,
    aio: Arc<Option<Aio>>,
}

impl Listener {
    pub(crate) fn new(listener: ssh2::Listener, aio: Arc<Option<Aio>>) -> Self {
        Self {
            inner: listener,
            aio,
        }
    }

    /// Accept a queued connection from this listener.
    pub async fn accept(&mut self) -> Result<Channel, Error> {
        let aio = self.aio.clone();
        let channel = into_the_future!(aio; &mut || { self.inner.accept() })?;
        Ok(Channel::new(channel, self.aio.clone()))
    }
}
