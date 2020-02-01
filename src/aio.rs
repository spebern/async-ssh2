use crate::BlockDirections;
use ssh2::Session;
use std::{
    cell::UnsafeCell,
    io,
    task::{Context, Poll},
};
use tokio::net::TcpStream;

pub struct Aio {
    stream: UnsafeCell<TcpStream>,
    session: Session,
}

unsafe impl Send for Aio {}
unsafe impl Sync for Aio {}

impl Aio {
    pub fn new(stream: TcpStream, session: Session) -> Self {
        Self {
            stream: UnsafeCell::new(stream),
            session,
        }
    }

    pub fn poll(&self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // This is safe because ssh2 locks whenever the underlying fd/socket is
        // used.
        let stream: &mut TcpStream = unsafe { &mut *self.stream.get() };

        match self.session.block_directions() {
            BlockDirections::Both => {
                if let Poll::Ready(_) = stream.poll_read_ready(cx) {
                    return Poll::Ready(Ok(()));
                }
                stream.poll_write_ready(cx)
            }
            BlockDirections::Inbound => stream.poll_read_ready(cx),
            BlockDirections::Outbound => stream.poll_write_ready(cx),
            BlockDirections::None => Poll::Ready(Ok(())),
        }
    }
}
