use crate::{channel::Channel, util::run_ssh2_fn, Error};
use smol::Async;
use ssh2::{self};
use std::{net::TcpStream, sync::Arc};

/// See [`Listener`](ssh2::Listener).
pub struct Listener {
    inner: ssh2::Listener,
    stream: Arc<Async<TcpStream>>,
}

impl Listener {
    pub(crate) fn new(listener: ssh2::Listener, stream: Arc<Async<TcpStream>>) -> Self {
        Self {
            inner: listener,
            stream,
        }
    }

    /// See [`accept`](ssh2::Listener::accept).
    pub async fn accept(&mut self) -> Result<Channel, Error> {
        let channel = run_ssh2_fn(&self.stream.clone(), || self.inner.accept()).await?;
        Ok(Channel::new(channel, self.stream.clone()))
    }
}
