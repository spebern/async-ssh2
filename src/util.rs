use crate::Error;
use async_io::Async;
use std::{io, 
    net::TcpStream,
    task::{Context, Poll},
};
use futures::{future, ready};
use futures_util;
use ssh2::{self, BlockDirections, ErrorCode};
use libssh2_sys;

fn would_block(e: &ssh2::Error) -> bool {
    match e.code() {
        ErrorCode::Session(e) if e == libssh2_sys::LIBSSH2_ERROR_EAGAIN => true,
        _ => false
    }
}

pub async fn run_ssh2_fn<R, F: FnMut() -> Result<R, ssh2::Error>>(
    stream: &Async<TcpStream>,
    session: &ssh2::Session,
    mut cb: F,
) -> Result<R, Error> {

    loop {
        match cb() {
            Ok(v) => return Ok(v),
            Err(e) if would_block(&e) => {
                match session.block_directions() {
                    BlockDirections::Inbound => {
                        stream.readable().await?
                    },
                    BlockDirections::Outbound => {
                        stream.writable().await?
                    },
                    BlockDirections::Both => {
                        let readable = stream.readable();
                        let writable = stream.writable();
                        futures_util::pin_mut!(readable);
                        futures_util::pin_mut!(writable);
                        let (ready,_) = future::select(readable, writable).await.factor_first();
                        ready?
                    },
                    BlockDirections::None => {
                        // This should not happen - libssh2 has already reported that it would block
                        panic!("libssh2 reports EAGAIN but is not blocked");
                    },
                }
            },
            Err(e) => return Err(Error::from(e))
        }
    }
}

/// Perform libssh2 asynchronous I/O Operation
pub fn poll_ssh2_io_op<T, F: FnMut() -> Result<T,io::Error>>(
    cx: &mut Context<'_>,
    stream: &Async<TcpStream>,
    session: &ssh2::Session,
    mut op: F,
) -> Poll<Result<T,io::Error>> {

    loop {
        match op() {
            Ok(result) => return Poll::Ready(Ok(result)),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                match session.block_directions() {
                    BlockDirections::Inbound => {
                        ready!(stream.poll_readable(cx))?;
                    },
                    BlockDirections::Outbound => {
                        ready!(stream.poll_writable(cx))?;
                    },
                    BlockDirections::Both => {
                        match stream.poll_readable(cx) {
                            Poll::Pending => ready!(stream.poll_writable(cx))?,
                            Poll::Ready(_) => {}
                        };
                    },
                    BlockDirections::None => {
                        // This should not happen - libssh2 has already reported that it would block
                        panic!("libssh2 reports EAGAIN but is not blocked");
                    },
                }
            },
            Err(e) => return Poll::Ready(Err(e))
        }
    }
}
