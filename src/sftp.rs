use crate::{util::{run_ssh2_fn,poll_ssh2_io_op},Error};
use futures::prelude::*;
use async_io::Async;
use ssh2::{self, FileStat, OpenFlags, OpenType};
use std::{
    io::{self, Read, Seek, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// See [`Sftp`](ssh2::Sftp).
pub struct Sftp {
    inner: ssh2::Sftp,
    inner_session: ssh2::Session,
    stream: Arc<Async<TcpStream>>,
}

/// See [`File`](ssh2::File).
pub struct File {
    inner: ssh2::File,
    inner_session: ssh2::Session,
    stream: Arc<Async<TcpStream>>,
}

impl Sftp {
    pub(crate) fn new<'b>(sftp: ssh2::Sftp, session: ssh2::Session, stream: Arc<Async<TcpStream>>) -> Sftp {
        Sftp {
            inner: sftp,
            inner_session: session,
            stream,
        }
    }

    /// See [`open_mode`](ssh2::Sftp::open_mode).
    pub async fn open_mode(
        &self,
        filename: &Path,
        flags: ssh2::OpenFlags,
        mode: i32,
        open_type: ssh2::OpenType,
    ) -> Result<File, Error> {
        let file = run_ssh2_fn(&self.stream, &self.inner_session,|| {
            self.inner.open_mode(filename, flags, mode, open_type)
        })
        .await?;
        Ok(File::new(file, self.inner_session.clone(), self.stream.clone()))
    }

    /// See [`open`](ssh2::Sftp::open).
    pub async fn open(&self, filename: &Path) -> Result<File, Error> {
        self.open_mode(filename, OpenFlags::READ, 0o644, OpenType::File)
            .await
    }

    /// See [`create`](ssh2::Sftp::create).
    pub async fn create(&self, filename: &Path) -> Result<File, Error> {
        self.open_mode(
            filename,
            OpenFlags::WRITE | OpenFlags::TRUNCATE,
            0o644,
            OpenType::File,
        )
        .await
    }

    /// See [`opendir`](ssh2::Sftp::opendir).
    pub async fn opendir(&self, dirname: &Path) -> Result<File, Error> {
        self.open_mode(dirname, OpenFlags::READ, 0, OpenType::Dir)
            .await
    }

    /// See [`readdir`](ssh2::Sftp::readdir).
    pub async fn readdir(&self, dirname: &Path) -> Result<Vec<(PathBuf, FileStat)>, Error> {
        let mut dir = self.opendir(dirname).await?;
        let mut ret = Vec::new();
        loop {
            match dir.readdir().await {
                Ok((filename, stat)) => {
                    if &*filename == Path::new(".") || &*filename == Path::new("..") {
                        continue;
                    }

                    ret.push((dirname.join(&filename), stat))
                }
                Err(Error::SSH2(ref e)) if e.code() == ssh2::ErrorCode::Session(-16) => {
                    break;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(ret)
    }

    /// See [`mkdir`](ssh2::Sftp::mkdir).
    pub async fn mkdir(&self, filename: &Path, mode: i32) -> Result<(), Error> {
        run_ssh2_fn(&self.stream, &self.inner_session,|| self.inner.mkdir(filename, mode)).await
    }

    /// See [`rmdir`](ssh2::Sftp::rmdir).
    pub async fn rmdir(&self, filename: &Path) -> Result<(), Error> {
        run_ssh2_fn(&self.stream, &self.inner_session,|| self.inner.rmdir(filename)).await
    }

    /// See [`stat`](ssh2::Sftp::stat).
    pub async fn stat(&self, filename: &Path) -> Result<ssh2::FileStat, Error> {
        run_ssh2_fn(&self.stream, &self.inner_session,|| self.inner.stat(filename)).await
    }

    /// See [`lstat`](ssh2::Sftp::lstat).
    pub async fn lstat(&self, filename: &Path) -> Result<ssh2::FileStat, Error> {
        run_ssh2_fn(&self.stream, &self.inner_session,|| self.inner.lstat(filename)).await
    }

    /// See [`setstat`](ssh2::Sftp::setstat).
    pub async fn setstat(&self, filename: &Path, stat: ssh2::FileStat) -> Result<(), Error> {
        run_ssh2_fn(&self.stream, &self.inner_session,|| self.inner.setstat(filename, stat.clone())).await
    }

    /// See [`symlink`](ssh2::Sftp::symlink).
    pub async fn symlink(&self, path: &Path, target: &Path) -> Result<(), Error> {
        run_ssh2_fn(&self.stream, &self.inner_session,|| self.inner.symlink(path, target)).await
    }

    /// See [`readlink`](ssh2::Sftp::readlink).
    pub async fn readlink(&self, path: &Path) -> Result<PathBuf, Error> {
        run_ssh2_fn(&self.stream, &self.inner_session,|| self.inner.readlink(path)).await
    }

    /// See [`realpath`](ssh2::Sftp::realpath).
    pub async fn realpath(&self, path: &Path) -> Result<PathBuf, Error> {
        run_ssh2_fn(&self.stream, &self.inner_session,|| self.inner.realpath(path)).await
    }

    /// See [`rename`](ssh2::Sftp::rename).
    pub async fn rename(
        &self,
        src: &Path,
        dst: &Path,
        flags: Option<ssh2::RenameFlags>,
    ) -> Result<(), Error> {
        run_ssh2_fn(&self.stream, &self.inner_session,|| self.inner.rename(src, dst, flags)).await
    }

    /// See [`unlink`](ssh2::Sftp::unlink).
    pub async fn unlink(&self, file: &Path) -> Result<(), Error> {
        run_ssh2_fn(&self.stream, &self.inner_session,|| self.inner.unlink(file)).await
    }

    /// See [`unlink`](ssh2::Sftp::shutdown).
    /// FIXME: This does not work properly. The inner `shutdown()` method can only be called once.
    /// When called it unwraps the sftp handle and calls libssh2_sftp_shutdown, which will likely return EAGAIN,
    /// but when we try to call it a second time it fails because the handle is already unwrapped.
    pub async fn shutdown(mut self) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), &self.inner_session.clone(), || self.inner.shutdown()).await
    }
}

impl File {
    pub(crate) fn new(file: ssh2::File, session: ssh2::Session, stream: Arc<Async<TcpStream>>) -> File {
        File {
            inner: file,
            inner_session: session,
            stream,
        }
    }

    /// See [`setstat`](ssh2::File::setstat).
    pub async fn setstat(&mut self, stat: FileStat) -> Result<(), Error> {
        let inner = &mut self.inner;
        run_ssh2_fn(&self.stream,  &self.inner_session, || inner.setstat(stat.clone())).await
    }

    /// See [`stat`](ssh2::File::stat).
    pub async fn stat(&mut self) -> Result<FileStat, Error> {
        let inner = &mut self.inner;
        run_ssh2_fn(&self.stream,  &self.inner_session, || inner.stat()).await
    }

    #[allow(missing_docs)]
    /// See [`statvfs`](ssh2::File::statvfs).
    // TODO
    /*
    pub async fn statvfs(&mut self) -> Result<raw::LIBSSH2_SFTP_STATVFS, Error> {
        run_ssh2_fn(&self.stream.clone(),  self.inner_session, || self.inner.statvfs().await
    }
    */

    /// See [`readdir`](ssh2::File::readdir).
    pub async fn readdir(&mut self) -> Result<(PathBuf, FileStat), Error> {
        let inner = &mut self.inner;
        run_ssh2_fn(&self.stream,  &self.inner_session, || inner.readdir()).await
    }

    /// See [`fsync`](ssh2::File::fsync).
    pub async fn fsync(&mut self) -> Result<(), Error> {
        let inner = &mut self.inner;
        run_ssh2_fn(&self.stream,  &self.inner_session, || inner.fsync()).await
    }

    /// See [`close`](ssh2::File::close).
    pub async fn close(mut self) -> Result<(), Error> {
        let inner = &mut self.inner;
        run_ssh2_fn(&self.stream,  &self.inner_session, || inner.close()).await
    }
}

impl AsyncRead for File {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let inner = &mut this.inner;
        poll_ssh2_io_op(cx, &this.stream.clone(), &this.inner_session, || inner.read(buf))
    }
}

impl AsyncWrite for File {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let this = self.get_mut();
        let inner = &mut this.inner;
        poll_ssh2_io_op(cx, &this.stream, &this.inner_session, || inner.write(buf))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let this = self.get_mut();
        let inner = &mut this.inner;
        poll_ssh2_io_op(cx, &this.stream, &this.inner_session, || inner.flush())
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let inner = &mut this.inner;
        poll_ssh2_io_op(cx, 
            &this.stream,
            &this.inner_session, 
            || inner.close().map_err(|e| io::Error::from(ssh2::Error::from_errno(e.code())))
        )
    }
}

impl Seek for File {
    fn seek(&mut self, pos: io::SeekFrom) -> Result<u64, io::Error> {
        self.inner.seek(pos)
    }
}
