use mio::{event::Evented, unix::EventedFd, Poll, PollOpt, Ready, Token};
use std::{io, os::unix::io::RawFd};

pub struct Io {
    fd: RawFd,
}

impl Io {
    pub fn new(fd: RawFd) -> Self {
        Self { fd }
    }
}

impl Evented for Io {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.fd).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.fd).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.fd).deregister(poll)
    }
}
