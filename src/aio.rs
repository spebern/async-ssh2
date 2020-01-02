use crate::{BlockDirections, Error};
use mio::{net::TcpStream, Ready};
use ssh2::Session;
use std::{io, task::Context};
use tokio::io::PollEvented;

pub struct Aio {
    poll_evented: PollEvented<TcpStream>,
    session: Session,
}

impl Aio {
    pub fn new(stream: std::net::TcpStream, session: Session) -> Result<Self, Error> {
        Ok(Self {
            poll_evented: PollEvented::new(TcpStream::from_stream(stream)?)?,
            session,
        })
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
