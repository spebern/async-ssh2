use crate::Error;
use smol::Async;
use std::{
    future::Future,
    io,
    net::TcpStream,
    task::{Context, Poll},
};

pub async fn run_ssh2_fn<R, F: FnMut() -> Result<R, ssh2::Error>>(
    stream: &Async<TcpStream>,
    mut cb: F,
) -> Result<R, Error> {
    let res = stream
        .read_with(|_s| match cb() {
            Ok(v) => Ok(Ok(v)),
            Err(e)
                if io::Error::from(ssh2::Error::from_errno(e.code())).kind()
                    == io::ErrorKind::WouldBlock =>
            {
                Err(io::Error::new(io::ErrorKind::WouldBlock, e))
            }
            Err(e) => Ok(Err(e)),
        })
        .await??;
    Ok(res)
}

pub(crate) fn poll_once<T>(cx: &mut Context<'_>, fut: impl Future<Output = T>) -> Poll<T> {
    pin_utils::pin_mut!(fut);
    fut.poll(cx)
}
