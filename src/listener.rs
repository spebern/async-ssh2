use crate::{channel::Channel, util::run_ssh2_fn, Error};
use async_io::Async;
use ssh2::{self};
use std::{net::TcpStream, sync::Arc};

/// See [`Listener`](ssh2::Listener).
pub struct Listener {
    inner: ssh2::Listener,
    inner_session: ssh2::Session,
    stream: Arc<Async<TcpStream>>,
}

impl Listener {
    pub(crate) fn new(listener: ssh2::Listener, session: ssh2::Session, stream: Arc<Async<TcpStream>>) -> Listener {
        Listener {
            inner: listener,
            inner_session: session,
            stream,
        }
    }

    /// See [`accept`](ssh2::Listener::accept).
    pub async fn accept(&mut self) -> Result<Channel, Error> {
        let inner = &mut self.inner;
        let channel = run_ssh2_fn(&self.stream.clone(), &self.inner_session, || inner.accept()).await?;
        Ok(Channel::new(channel, self.inner_session.clone(), self.stream.clone()))
    }
}
