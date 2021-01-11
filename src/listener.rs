use crate::{channel::Channel, util::run_ssh2_fn, Error};
use async_io::Async;
use ssh2::{self};
use std::{net::TcpStream, sync::Arc};

/// See [`Listener`](ssh2::Listener).
pub struct Listener<'a> {
    inner: ssh2::Listener,
    inner_session: &'a ssh2::Session,
    stream: Arc<Async<TcpStream>>,
}

impl<'a> Listener<'a> {
    pub(crate) fn new<'b>(listener: ssh2::Listener, session: &'b ssh2::Session, stream: Arc<Async<TcpStream>>) -> Listener<'b> {
        Listener {
            inner: listener,
            inner_session: session,
            stream,
        }
    }

    /// See [`accept`](ssh2::Listener::accept).
    pub async fn accept<'b>(&'b mut self) -> Result<Channel<'b>, Error> {
        let channel = run_ssh2_fn(&self.stream.clone(), self.inner_session, || self.inner.accept()).await?;
        Ok(Channel::new(channel, self.inner_session, self.stream.clone()))
    }
}
