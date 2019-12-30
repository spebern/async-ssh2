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

/// A channel represents a portion of an SSH connection on which data can be
/// read and written.
///
/// Channels denote all of SCP uploads and downloads, shell sessions, remote
/// process executions, and other general-purpose sessions. Each channel
/// implements the `Reader` and `Writer` traits to send and receive data.
/// Whether or not I/O operations are blocking is mandated by the `blocking`
/// flag on a channel's corresponding `Session`.
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

    /// Set an environment variable in the remote channel's process space.
    ///
    /// Note that this does not make sense for all channel types and may be
    /// ignored by the server despite returning success.
    pub async fn setenv(&mut self, var: &str, val: &str) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.setenv(var, val) })
    }

    /// Request a PTY on an established channel.
    ///
    /// Note that this does not make sense for all channel types and may be
    /// ignored by the server despite returning success.
    ///
    /// The dimensions argument is a tuple of (width, height, width_px,
    /// height_px)
    ///
    /// The mode parameter is optional and specifies modes to apply to
    /// the pty.  Use the `PtyModes` type construct these modes.
    /// A contrived example of this is below:
    ///
    /// ```
    /// let mut mode = ssh2::PtyModes::new();
    /// // Set the interrupt character to CTRL-C (ASCII 3: ETX).
    /// // This is typically the default, but we're showing how to
    /// // set a relatable option for the sake of example!
    /// mode.set_character(ssh2::PtyModeOpcode::VINTR, Some(3 as char));
    /// ```
    pub async fn request_pty(
        &mut self,
        term: &str,
        mode: Option<PtyModes>,
        dim: Option<(u32, u32, u32, u32)>,
    ) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.request_pty(term, mode.clone(), dim) })
    }

    /// Request that the PTY size be changed to the specified size.
    /// width and height are the number of character cells, and you
    /// may optionally include the size specified in pixels.
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

    /// Execute a command
    ///
    /// An execution is one of the standard process services defined by the SSH2
    /// protocol.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::io::prelude::*;
    /// # use ssh2::Session;
    /// # let session: Session = panic!();
    /// let mut channel = session.channel_session().unwrap();
    /// channel.exec("ls").unwrap();
    /// let mut s = String::new();
    /// channel.read_to_string(&mut s).unwrap();
    /// println!("{}", s);
    /// ```
    pub async fn exec(&mut self, command: &str) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.exec(command) })
    }

    /// Start a shell
    ///
    /// A shell is one of the standard process services defined by the SSH2
    /// protocol.
    pub async fn shell(&mut self) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.shell() })
    }

    /// Request a subsystem be started.
    ///
    /// A subsystem is one of the standard process services defined by the SSH2
    /// protocol.
    pub async fn subsystem(&mut self, system: &str) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.subsystem(system) })
    }

    /// Initiate a request on a session type channel.
    ///
    /// The SSH2 protocol currently defines shell, exec, and subsystem as
    /// standard process services.
    pub async fn process_startup(
        &mut self,
        request: &str,
        message: Option<&str>,
    ) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.process_startup(request, message) })
    }

    /// Get a handle to the stderr stream of this channel.
    ///
    /// The returned handle implements the `Read` and `Write` traits.
    pub fn stderr(&mut self) -> Stream<'_> {
        self.inner.stderr()
    }

    /// Get a handle to a particular stream for this channel.
    ///
    /// The returned handle implements the `Read` and `Write` traits.
    ///
    /// Groups of substreams may be flushed by passing on of the following
    /// constants and then calling `flush()`.
    ///
    /// * FLUSH_EXTENDED_DATA - Flush all extended data substreams
    /// * FLUSH_ALL - Flush all substreams
    pub fn stream<'a>(&'a mut self, stream_id: i32) -> Stream<'a> {
        self.inner.stream(stream_id)
    }

    /// Change how extended data (such as stderr) is handled
    pub async fn handle_extended_data(&mut self, mode: ExtendedData) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.handle_extended_data(mode) })
    }

    /// Returns the exit code raised by the process running on the remote host
    /// at the other end of the named channel.
    ///
    /// Note that the exit status may not be available if the remote end has not
    /// yet set its status to closed.
    pub fn exit_status(&self) -> Result<i32, Error> {
        self.inner.exit_status()
    }

    /// Get the remote exit signal.
    pub fn exit_signal(&self) -> Result<ExitSignal, Error> {
        self.inner.exit_signal()
    }

    /// Check the status of the read window.
    pub fn read_window(&self) -> ReadWindow {
        self.inner.read_window()
    }

    /// Check the status of the write window.
    pub fn write_window(&self) -> WriteWindow {
        self.inner.write_window()
    }

    /// Adjust the receive window for a channel by adjustment bytes.
    ///
    /// If the amount to be adjusted is less than the minimum adjustment and
    /// force is false, the adjustment amount will be queued for a later packet.
    ///
    /// This function returns the new size of the receive window (as understood
    /// by remote end) on success.
    pub async fn adjust_receive_window(&mut self, adjust: u64, force: bool) -> Result<u64, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.adjust_receive_window(adjust, force) })
    }

    /// Artificially limit the number of bytes that will be read from this
    /// channel. Hack intended for use by scp_recv only.
    #[doc(hidden)]
    pub fn limit_read(&mut self, limit: u64) {
        self.inner.limit_read(limit)
    }

    /// Check if the remote host has sent an EOF status for the channel.
    /// Take care: the EOF status is for the entire channel which can be confusing
    /// because the reading from the channel reads only the stdout stream.
    /// unread, buffered, stderr data will cause eof() to return false.
    pub fn eof(&self) -> bool {
        self.inner.eof()
    }

    /// Tell the remote host that no further data will be sent on the specified
    /// channel.
    ///
    /// Processes typically interpret this as a closed stdin descriptor.
    pub async fn send_eof(&mut self) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.send_eof() })
    }

    /// Wait for the remote end to send EOF.
    /// Note that unread buffered stdout and stderr will cause this function
    /// to return `Ok(())` without waiting.
    /// You should call the eof() function after calling this to check the
    /// status of the channel.
    pub async fn wait_eof(&mut self) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.wait_eof() })
    }

    /// Close an active data channel.
    ///
    /// In practice this means sending an SSH_MSG_CLOSE packet to the remote
    /// host which serves as instruction that no further data will be sent to
    /// it. The remote host may still send data back until it sends its own
    /// close message in response.
    ///
    /// To wait for the remote end to close its connection as well, follow this
    /// command with `wait_closed`
    pub async fn close(&mut self) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.close() })
    }

    /// Enter a temporary blocking state until the remote host closes the named
    /// channel.
    ///
    /// Typically sent after `close` in order to examine the exit status.
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
