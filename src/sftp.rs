use crate::{aio::Aio, into_the_future};
use libc::c_int;
use ssh2::{self, Error, FileStat};
use std::{
    future::Future,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite};

/// See [`Sftp`](ssh2::Sftp).
pub struct Sftp {
    inner: ssh2::Sftp,
    aio: Arc<Option<Aio>>,
}

/// See [`File`](ssh2::File).
pub struct File<'sftp> {
    inner: ssh2::File<'sftp>,
    aio: Arc<Option<Aio>>,
}

impl Sftp {
    pub(crate) fn new(sftp: ssh2::Sftp, aio: Arc<Option<Aio>>) -> Self {
        Self { inner: sftp, aio }
    }

    /// See [`open_mode`](ssh2::Sftp::open_mode).
    pub async fn open_mode(
        &self,
        filename: &Path,
        flags: ssh2::OpenFlags,
        mode: i32,
        open_type: ssh2::OpenType,
    ) -> Result<File<'_>, Error> {
        let aio = self.aio.clone();
        let file = into_the_future!(aio; &mut || { self.inner.open_mode(filename, flags, mode, open_type) })?;
        Ok(File::new(file, self.aio.clone()))
    }

    /// See [`open`](ssh2::Sftp::open).
    pub async fn open(&self, filename: &Path) -> Result<File<'_>, Error> {
        let aio = self.aio.clone();
        let file = into_the_future!(aio; &mut || { self.inner.open(filename) })?;
        Ok(File::new(file, self.aio.clone()))
    }

    /// See [`create`](ssh2::Sftp::create).
    pub async fn create(&self, filename: &Path) -> Result<File<'_>, Error> {
        let aio = self.aio.clone();
        let file = into_the_future!(aio; &mut || { self.inner.create(filename) })?;
        Ok(File::new(file, self.aio.clone()))
    }

    /// See [`opendir`](ssh2::Sftp::opendir).
    pub async fn opendir(&self, dirname: &Path) -> Result<File<'_>, Error> {
        let aio = self.aio.clone();
        let file = into_the_future!(aio; &mut || { self.inner.opendir(dirname) })?;
        Ok(File::new(file, self.aio.clone()))
    }

    /// See [`readdir`](ssh2::Sftp::readdir).
    pub async fn readdir(&self, dirname: &Path) -> Result<Vec<(PathBuf, ssh2::FileStat)>, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.readdir(dirname) })
    }

    /// See [`mkdir`](ssh2::Sftp::mkdir).
    pub async fn mkdir(&self, filename: &Path, mode: i32) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.mkdir(filename, mode) })
    }

    /// See [`rmdir`](ssh2::Sftp::rmdir).
    pub async fn rmdir(&self, filename: &Path) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.rmdir(filename) })
    }

    /// See [`stat`](ssh2::Sftp::stat).
    pub async fn stat(&self, filename: &Path) -> Result<ssh2::FileStat, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.stat(filename) })
    }

    /// See [`lstat`](ssh2::Sftp::lstat).
    pub async fn lstat(&self, filename: &Path) -> Result<ssh2::FileStat, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.lstat(filename) })
    }

    /// See [`setstat`](ssh2::Sftp::setstat).
    pub async fn setstat(&self, filename: &Path, stat: ssh2::FileStat) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.setstat(filename, stat.clone()) })
    }

    /// See [`symlink`](ssh2::Sftp::symlink).
    pub async fn symlink(&self, path: &Path, target: &Path) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.symlink(path, target) })
    }

    /// See [`readlink`](ssh2::Sftp::readlink).
    pub async fn readlink(&self, path: &Path) -> Result<PathBuf, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.readlink(path) })
    }

    /// See [`realpath`](ssh2::Sftp::realpath).
    pub async fn realpath(&self, path: &Path) -> Result<PathBuf, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.realpath(path) })
    }

    /// See [`rename`](ssh2::Sftp::rename).
    pub async fn rename(
        &self,
        src: &Path,
        dst: &Path,
        flags: Option<ssh2::RenameFlags>,
    ) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.rename(src, dst, flags) })
    }

    /// See [`unlink`](ssh2::Sftp::unlink).
    pub async fn unlink(&self, file: &Path) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.unlink(file) })
    }

    /// See [`last_error`](ssh2::Sftp::last_error).
    pub fn last_error(&self) -> Error {
        self.inner.last_error()
    }

    /// See [`rc`](ssh2::Sftp::rc).
    pub fn rc(&self, rc: c_int) -> Result<(), Error> {
        self.inner.rc(rc)
    }
}

impl<'sftp> File<'sftp> {
    pub(crate) fn new(file: ssh2::File<'sftp>, aio: Arc<Option<Aio>>) -> Self {
        Self { inner: file, aio }
    }

    /// See [`setstat`](ssh2::File::setstat).
    pub async fn setstat(&mut self, stat: FileStat) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.setstat(stat.clone()) })
    }

    /// See [`stat`](ssh2::File::stat).
    pub async fn stat(&mut self) -> Result<FileStat, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.stat() })
    }

    #[allow(missing_docs)]
    /// See [`statvfs`](ssh2::File::statvfs).
    // TODO
    /*
    pub async fn statvfs(&mut self) -> Result<raw::LIBSSH2_SFTP_STATVFS, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.statvfs() })
    }
    */

    /// See [`readdir`](ssh2::File::readdir).
    pub async fn readdir(&mut self) -> Result<(PathBuf, FileStat), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.readdir() })
    }

    /// See [`fsync`](ssh2::File::fsync).
    pub async fn fsync(&mut self) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.fsync() })
    }
}

impl<'sftp> AsyncRead for File<'sftp> {
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

impl<'sftp> AsyncWrite for File<'sftp> {
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
