use crate::{aio::Aio, channel::Channel, Error};
use ssh2::{self};
use std::{
    future::Future,
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// See [`Listener`](ssh2::Listener).
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

    /// See [`accept`](ssh2::Listener::accept).
    pub async fn accept(&mut self) -> Result<Channel, Error> {
        let aio = self.aio.clone();
        let channel = into_the_future!(aio; &mut || { self.inner.accept() })?;
        Ok(Channel::new(channel, self.aio.clone()))
    }
}
