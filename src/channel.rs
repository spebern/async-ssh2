use crate::{aio::Aio, into_the_future};

use ssh2::{self, Error, ExitSignal, ExtendedData, PtyModes, ReadWindow, Stream, WriteWindow};
use std::{
    future::Future,
    io,
    io::{Read, Write},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite};

/// See [`Channel`](ssh2::Channel).
pub struct Channel {
    inner: ssh2::Channel,
    aio: Arc<Option<Aio>>,
}

impl Channel {
    pub(crate) fn new(channel: ssh2::Channel, aio: Arc<Option<Aio>>) -> Self {
        Self {
            inner: channel,
            aio,
        }
    }

    /// See [`setenv`](ssh2::Channel::setenv).
    pub async fn setenv(&mut self, var: &str, val: &str) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.setenv(var, val) })
    }

    /// See [`request_pty`](ssh2::Channel::request_pty).
    pub async fn request_pty(
        &mut self,
        term: &str,
        mode: Option<PtyModes>,
        dim: Option<(u32, u32, u32, u32)>,
    ) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.request_pty(term, mode.clone(), dim) })
    }

    /// See [`request_pty_size`](ssh2::Channel::request_pty_size).
    pub async fn request_pty_size(
        &mut self,
        width: u32,
        height: u32,
        width_px: Option<u32>,
        height_px: Option<u32>,
    ) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.request_pty_size(width, height, width_px, height_px) })
    }

    /// See [`exec`](ssh2::Channel::exec).
    pub async fn exec(&mut self, command: &str) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.exec(command) })
    }

    /// See [`shell`](ssh2::Channel::shell).
    pub async fn shell(&mut self) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.shell() })
    }

    /// See [`subsystem`](ssh2::Channel::subsystem).
    pub async fn subsystem(&mut self, system: &str) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.subsystem(system) })
    }

    /// See [`process_startup`](ssh2::Channel::process_startup).
    pub async fn process_startup(
        &mut self,
        request: &str,
        message: Option<&str>,
    ) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.process_startup(request, message) })
    }

    /// See [`stderr`](ssh2::Channel::stderr).
    pub fn stderr(&mut self) -> Stream<'_> {
        self.inner.stderr()
    }

    /// See [`stream`](ssh2::Channel::stream).
    pub fn stream(&mut self, stream_id: i32) -> Stream<'_> {
        self.inner.stream(stream_id)
    }

    /// See [`handle_extended_data`](ssh2::Channel::handle_extended_data).
    pub async fn handle_extended_data(&mut self, mode: ExtendedData) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.handle_extended_data(mode) })
    }

    /// See [`exit_status`](ssh2::Channel::exit_status).
    pub fn exit_status(&self) -> Result<i32, Error> {
        self.inner.exit_status()
    }

    /// See [`exit_signal`](ssh2::Channel::exit_signal).
    pub fn exit_signal(&self) -> Result<ExitSignal, Error> {
        self.inner.exit_signal()
    }

    /// See [`read_window`](ssh2::Channel::read_window).
    pub fn read_window(&self) -> ReadWindow {
        self.inner.read_window()
    }

    /// See [`write_window`](ssh2::Channel::write_window).
    pub fn write_window(&self) -> WriteWindow {
        self.inner.write_window()
    }

    /// See [`adjust_receive_window`](ssh2::Channel::adjust_receive_window).
    pub async fn adjust_receive_window(&mut self, adjust: u64, force: bool) -> Result<u64, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.adjust_receive_window(adjust, force) })
    }

    /// See [`limit_read`](ssh2::Channel::limit_read).
    #[doc(hidden)]
    pub fn limit_read(&mut self, limit: u64) {
        self.inner.limit_read(limit)
    }

    /// See [`eof`](ssh2::Channel::eof).
    pub fn eof(&self) -> bool {
        self.inner.eof()
    }

    /// See [`send_eof`](ssh2::Channel::send_eof).
    pub async fn send_eof(&mut self) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.send_eof() })
    }

    /// See [`wait_eof`](ssh2::Channel::wait_eof).
    pub async fn wait_eof(&mut self) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.wait_eof() })
    }

    /// See [`close`](ssh2::Channel::close).
    pub async fn close(&mut self) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.close() })
    }

    /// See [`wait_close`](ssh2::Channel::wait_close).
    pub async fn wait_close(&mut self) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.wait_close() })
    }
}

impl AsyncRead for Channel {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            let res = self.inner.read(buf);
            match res {
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if let Some(ref aio) = *self.aio {
                        aio.set_waker(cx).unwrap();
                    }
                    return Poll::Pending;
                }
                Err(e) => return Poll::Ready(Err(e)),
                Ok(val) => return Poll::Ready(Ok(val)),
            }
        }
    }
}

impl AsyncWrite for Channel {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        loop {
            let res = self.inner.write(buf);
            match res {
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if let Some(ref aio) = *self.aio {
                        aio.set_waker(cx).unwrap();
                    }
                    return Poll::Pending;
                }
                Err(e) => return Poll::Ready(Err(e)),
                Ok(val) => return Poll::Ready(Ok(val)),
            }
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        loop {
            let res = self.inner.flush();
            match res {
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if let Some(ref aio) = *self.aio {
                        aio.set_waker(cx).unwrap();
                    }
                    return Poll::Pending;
                }
                Err(e) => return Poll::Ready(Err(e)),
                Ok(val) => return Poll::Ready(Ok(val)),
            }
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(().into()))
    }
}

/*
impl<'channel> Read for Stream<'channel> {
    fn read(&mut self, data: &mut [u8]) -> io::Result<usize> {
        if self.channel.eof() {
            return Ok(0);
        }

        let data = match self.channel.read_limit {
            Some(amt) => {
                let len = data.len();
                &mut data[..cmp::min(amt as usize, len)]
            }
            None => data,
        };
        let ret = unsafe {
            let rc = raw::libssh2_channel_read_ex(
                self.channel.raw,
                self.id as c_int,
                data.as_mut_ptr() as *mut _,
                data.len() as size_t,
            );
            self.channel.sess.rc(rc as c_int).map(|()| rc as usize)
        };
        match ret {
            Ok(n) => {
                if let Some(ref mut amt) = self.channel.read_limit {
                    *amt -= n as u64;
                }
                Ok(n)
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl<'channel> Write for Stream<'channel> {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        unsafe {
            let rc = raw::libssh2_channel_write_ex(
                self.channel.raw,
                self.id as c_int,
                data.as_ptr() as *mut _,
                data.len() as size_t,
            );
            self.channel.sess.rc(rc as c_int).map(|()| rc as usize)
        }
        .map_err(Into::into)
    }

    fn flush(&mut self) -> io::Result<()> {
        unsafe {
            let rc = raw::libssh2_channel_flush_ex(self.channel.raw, self.id as c_int);
            self.channel.sess.rc(rc)
        }
        .map_err(Into::into)
    }
}
*/
