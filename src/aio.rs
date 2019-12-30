use crate::BlockDirections;
use mio::{event::Evented, unix::EventedFd, Poll, PollOpt, Ready, Token};
use ssh2::Session;
use std::{io, os::unix, task::Context};
use tokio::io::PollEvented;

pub struct Aio {
    poll_evented: PollEvented<RawFd>,
    session: Session,
}

impl Aio {
    pub fn new(fd: unix::io::RawFd, session: Session) -> Self {
        Self {
            // TODO get rid of unwrap
            poll_evented: PollEvented::new(RawFd(fd)).unwrap(),
            session,
        }
    }

    pub fn set_waker(&self, ctx: &mut Context<'_>) -> io::Result<()> {
        match self.session.block_directions() {
            BlockDirections::Both => {
                self.poll_evented.clear_read_ready(ctx, Ready::readable())?;
                self.poll_evented.clear_write_ready(ctx)?;
            }
            BlockDirections::Inbound => {
                self.poll_evented.clear_read_ready(ctx, Ready::readable())?;
            }
            BlockDirections::Outbound => {
                self.poll_evented.clear_write_ready(ctx)?;
            }
            BlockDirections::None => {}
        }
        Ok(())
    }
}

struct RawFd(unix::io::RawFd);

impl Evented for RawFd {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.0).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.0).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.0).deregister(poll)
    }
}
