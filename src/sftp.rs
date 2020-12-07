use crate::{util::run_ssh2_fn, Error};
use futures::prelude::*;
use smol::Async;
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
    stream: Arc<Async<TcpStream>>,
}

/// See [`File`](ssh2::File).
pub struct File {
    inner: ssh2::File,
    stream: Arc<Async<TcpStream>>,
}

impl Sftp {
    pub(crate) fn new(sftp: ssh2::Sftp, stream: Arc<Async<TcpStream>>) -> Self {
        Self {
            inner: sftp,
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
        let file = run_ssh2_fn(&self.stream, || {
            self.inner.open_mode(filename, flags, mode, open_type)
        })
        .await?;
        Ok(File::new(file, self.stream.clone()))
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
                Err(Error::SSH2(ref e)) if e.code() == -16 => {
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
        run_ssh2_fn(&self.stream, || self.inner.mkdir(filename, mode)).await
    }

    /// See [`rmdir`](ssh2::Sftp::rmdir).
    pub async fn rmdir(&self, filename: &Path) -> Result<(), Error> {
        run_ssh2_fn(&self.stream, || self.inner.rmdir(filename)).await
    }

    /// See [`stat`](ssh2::Sftp::stat).
    pub async fn stat(&self, filename: &Path) -> Result<ssh2::FileStat, Error> {
        run_ssh2_fn(&self.stream, || self.inner.stat(filename)).await
    }

    /// See [`lstat`](ssh2::Sftp::lstat).
    pub async fn lstat(&self, filename: &Path) -> Result<ssh2::FileStat, Error> {
        run_ssh2_fn(&self.stream, || self.inner.lstat(filename)).await
    }

    /// See [`setstat`](ssh2::Sftp::setstat).
    pub async fn setstat(&self, filename: &Path, stat: ssh2::FileStat) -> Result<(), Error> {
        run_ssh2_fn(&self.stream, || self.inner.setstat(filename, stat.clone())).await
    }

    /// See [`symlink`](ssh2::Sftp::symlink).
    pub async fn symlink(&self, path: &Path, target: &Path) -> Result<(), Error> {
        run_ssh2_fn(&self.stream, || self.inner.symlink(path, target)).await
    }

    /// See [`readlink`](ssh2::Sftp::readlink).
    pub async fn readlink(&self, path: &Path) -> Result<PathBuf, Error> {
        run_ssh2_fn(&self.stream, || self.inner.readlink(path)).await
    }

    /// See [`realpath`](ssh2::Sftp::realpath).
    pub async fn realpath(&self, path: &Path) -> Result<PathBuf, Error> {
        run_ssh2_fn(&self.stream, || self.inner.realpath(path)).await
    }

    /// See [`rename`](ssh2::Sftp::rename).
    pub async fn rename(
        &self,
        src: &Path,
        dst: &Path,
        flags: Option<ssh2::RenameFlags>,
    ) -> Result<(), Error> {
        run_ssh2_fn(&self.stream, || self.inner.rename(src, dst, flags)).await
    }

    /// See [`unlink`](ssh2::Sftp::unlink).
    pub async fn unlink(&self, file: &Path) -> Result<(), Error> {
        run_ssh2_fn(&self.stream, || self.inner.unlink(file)).await
    }

    /// See [`unlink`](ssh2::Sftp::unlink).
    pub async fn shutdown(mut self) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), || self.inner.shutdown()).await
    }
}

impl File {
    pub(crate) fn new(file: ssh2::File, stream: Arc<Async<TcpStream>>) -> Self {
        Self {
            inner: file,
            stream,
        }
    }

    /// See [`setstat`](ssh2::File::setstat).
    pub async fn setstat(&mut self, stat: FileStat) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), || self.inner.setstat(stat.clone())).await
    }

    /// See [`stat`](ssh2::File::stat).
    pub async fn stat(&mut self) -> Result<FileStat, Error> {
        run_ssh2_fn(&self.stream.clone(), || self.inner.stat()).await
    }

    #[allow(missing_docs)]
    /// See [`statvfs`](ssh2::File::statvfs).
    // TODO
    /*
    pub async fn statvfs(&mut self) -> Result<raw::LIBSSH2_SFTP_STATVFS, Error> {
        run_ssh2_fn(&self.stream.clone(), || self.inner.statvfs().await
    }
    */

    /// See [`readdir`](ssh2::File::readdir).
    pub async fn readdir(&mut self) -> Result<(PathBuf, FileStat), Error> {
        run_ssh2_fn(&self.stream.clone(), || self.inner.readdir()).await
    }

    /// See [`fsync`](ssh2::File::fsync).
    pub async fn fsync(&mut self) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), || self.inner.fsync()).await
    }

    /// See [`close`](ssh2::File::close).
    pub async fn close(mut self) -> Result<(), Error> {
        run_ssh2_fn(&self.stream.clone(), || self.inner.close()).await
    }
}

impl AsyncRead for File {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.stream
            .clone()
            .read_with(|_s| self.inner.read(buf))
            .boxed()
            .poll_unpin(cx)
    }
}

impl AsyncWrite for File {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        self.stream
            .clone()
            .write_with(|_s| self.inner.write(buf))
            .boxed()
            .poll_unpin(cx)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.stream
            .clone()
            .write_with(|_s| self.inner.flush())
            .boxed()
            .poll_unpin(cx)
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

impl Seek for File {
    fn seek(&mut self, pos: io::SeekFrom) -> Result<u64, io::Error> {
        self.inner.seek(pos)
    }
}
