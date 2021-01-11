use crate::{util::{run_ssh2_fn, poll_ssh2_io_op}, Error};
use futures::prelude::*;
use async_io::Async;
use ssh2::{self, ExitSignal, ExtendedData, PtyModes, ReadWindow, Stream, WriteWindow};
use std::{
    convert::From,
    io,
    io::{Read, Write},
    net::TcpStream,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// See [`Channel`](ssh2::Channel).
pub struct Channel<'a> {
    inner: ssh2::Channel,
    inner_session: &'a ssh2::Session,
    stream: Arc<Async<TcpStream>>,
}

impl<'a> Channel<'a> {
    pub(crate) fn new<'b>(channel: ssh2::Channel, session: &'b ssh2::Session, stream: Arc<Async<TcpStream>>) -> Channel<'b> {
        Channel {
            inner: channel,
            inner_session: session,
            stream,
        }
    }

    /// See [`setenv`](ssh2::Channel::setenv).
    pub async fn setenv(&mut self, var: &str, val: &str) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), self.inner_session, || self.inner.setenv(var, val)).await
    }

    /// See [`request_pty`](ssh2::Channel::request_pty).
    pub async fn request_pty(
        &mut self,
        term: &str,
        mode: Option<PtyModes>,
        dim: Option<(u32, u32, u32, u32)>,
    ) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), self.inner_session, || {
            self.inner.request_pty(term, mode.clone(), dim)
        })
        .await
    }

    /// See [`request_pty_size`](ssh2::Channel::request_pty_size).
    pub async fn request_pty_size(
        &mut self,
        width: u32,
        height: u32,
        width_px: Option<u32>,
        height_px: Option<u32>,
    ) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), self.inner_session, || {
            self.inner
                .request_pty_size(width, height, width_px, height_px)
        })
        .await
    }

    /// See [`exec`](ssh2::Channel::exec).
    pub async fn exec(&mut self, command: &str) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), self.inner_session, || self.inner.exec(command)).await
    }

    /// See [`shell`](ssh2::Channel::shell).
    pub async fn shell(&mut self) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), self.inner_session, || self.inner.shell()).await
    }

    /// See [`subsystem`](ssh2::Channel::subsystem).
    pub async fn subsystem(&mut self, system: &str) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), self.inner_session, || self.inner.subsystem(system)).await
    }

    /// See [`process_startup`](ssh2::Channel::process_startup).
    pub async fn process_startup(
        &mut self,
        request: &str,
        message: Option<&str>,
    ) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), self.inner_session, || {
            self.inner.process_startup(request, message)
        })
        .await
    }

    /// See [`stderr`](ssh2::Channel::stderr).
    pub fn stderr(&mut self) -> Stream {
        self.inner.stderr()
    }

    /// See [`stream`](ssh2::Channel::stream).
    pub fn stream(&mut self, stream_id: i32) -> Stream {
        self.inner.stream(stream_id)
    }

    /// See [`handle_extended_data`](ssh2::Channel::handle_extended_data).
    pub async fn handle_extended_data(&mut self, mode: ExtendedData) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), self.inner_session, || {
            self.inner.handle_extended_data(mode)
        })
        .await
    }

    /// See [`exit_status`](ssh2::Channel::exit_status).
    pub fn exit_status(&self) -> Result<i32, Error> {
        self.inner.exit_status().map_err(From::from)
    }

    /// See [`exit_signal`](ssh2::Channel::exit_signal).
    pub fn exit_signal(&self) -> Result<ExitSignal, Error> {
        self.inner.exit_signal().map_err(From::from)
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
        run_ssh2_fn(&self.stream.clone(), self.inner_session, || {
            self.inner.adjust_receive_window(adjust, force)
        })
        .await
    }

    /// See [`eof`](ssh2::Channel::eof).
    pub fn eof(&self) -> bool {
        self.inner.eof()
    }

    /// See [`send_eof`](ssh2::Channel::send_eof).
    pub async fn send_eof(&mut self) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), self.inner_session, || self.inner.send_eof()).await
    }

    /// See [`wait_eof`](ssh2::Channel::wait_eof).
    pub async fn wait_eof(&mut self) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), self.inner_session, || self.inner.wait_eof()).await
    }

    /// See [`close`](ssh2::Channel::close).
    pub async fn close(&mut self) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), self.inner_session, || self.inner.close()).await
    }

    /// See [`wait_close`](ssh2::Channel::wait_close).
    pub async fn wait_close(&mut self) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), self.inner_session, || self.inner.wait_close()).await
    }
}

impl<'a> AsyncRead for Channel<'a> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        poll_ssh2_io_op(cx, &self.stream.clone(), &self.inner_session, || self.inner.read(buf))
    }
}

impl<'a> AsyncWrite for Channel<'a> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        poll_ssh2_io_op(cx, &self.stream.clone(), &self.inner_session, || self.inner.write(buf))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        poll_ssh2_io_op(cx, &self.stream.clone(), &self.inner_session, || self.inner.flush())
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        poll_ssh2_io_op(cx, 
            &self.stream.clone(), 
            &self.inner_session, 
            || self.inner.close().map_err(|e| io::Error::from(ssh2::Error::from_errno(e.code())))
        )
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
