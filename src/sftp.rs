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

/// A handle to a remote filesystem over SFTP.
///
/// Instances are created through the `sftp` method on a `Session`.
pub struct Sftp {
    inner: ssh2::Sftp,
    aio: Arc<Option<Aio>>,
}

/// A file handle to an SFTP connection.
///
/// Files behave similarly to `std::old_io::File` in that they are readable and
/// writable and support operations like stat and seek.
///
/// Files are created through `open`, `create`, and `open_mode` on an instance
/// of `Sftp`.
pub struct File<'sftp> {
    inner: ssh2::File<'sftp>,
    aio: Arc<Option<Aio>>,
}

impl Sftp {
    pub(crate) fn new(sftp: ssh2::Sftp, aio: Arc<Option<Aio>>) -> Self {
        Self { inner: sftp, aio }
    }

    /// Open a handle to a file.
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

    /// Helper to open a file in the `Read` mode.
    pub async fn open(&self, filename: &Path) -> Result<File<'_>, Error> {
        let aio = self.aio.clone();
        let file = into_the_future!(aio; &mut || { self.inner.open(filename) })?;
        Ok(File::new(file, self.aio.clone()))
    }

    /// Helper to create a file in write-only mode with truncation.
    pub async fn create(&self, filename: &Path) -> Result<File<'_>, Error> {
        let aio = self.aio.clone();
        let file = into_the_future!(aio; &mut || { self.inner.create(filename) })?;
        Ok(File::new(file, self.aio.clone()))
    }

    /// Helper to open a directory for reading its contents.
    pub async fn opendir(&self, dirname: &Path) -> Result<File<'_>, Error> {
        let aio = self.aio.clone();
        let file = into_the_future!(aio; &mut || { self.inner.opendir(dirname) })?;
        Ok(File::new(file, self.aio.clone()))
    }

    /// Convenience function to read the files in a directory.
    ///
    /// The returned paths are all joined with `dirname` when returned, and the
    /// paths `.` and `..` are filtered out of the returned list.
    pub async fn readdir(&self, dirname: &Path) -> Result<Vec<(PathBuf, ssh2::FileStat)>, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.readdir(dirname) })
    }

    /// Create a directory on the remote file system.
    pub async fn mkdir(&self, filename: &Path, mode: i32) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.mkdir(filename, mode) })
    }

    /// Remove a directory from the remote file system.
    pub async fn rmdir(&self, filename: &Path) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.rmdir(filename) })
    }

    /// Get the metadata for a file, performed by stat(2)
    pub async fn stat(&self, filename: &Path) -> Result<ssh2::FileStat, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.stat(filename) })
    }

    /// Get the metadata for a file, performed by lstat(2)
    pub async fn lstat(&self, filename: &Path) -> Result<ssh2::FileStat, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.lstat(filename) })
    }

    /// Set the metadata for a file.
    pub async fn setstat(&self, filename: &Path, stat: ssh2::FileStat) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.setstat(filename, stat.clone()) })
    }

    /// Create a symlink at `target` pointing at `path`.
    pub async fn symlink(&self, path: &Path, target: &Path) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.symlink(path, target) })
    }

    /// Read a symlink at `path`.
    pub async fn readlink(&self, path: &Path) -> Result<PathBuf, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.readlink(path) })
    }

    /// Resolve the real path for `path`.
    pub async fn realpath(&self, path: &Path) -> Result<PathBuf, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.realpath(path) })
    }

    /// Rename a filesystem object on the remote filesystem.
    ///
    /// The semantics of this command typically include the ability to move a
    /// filesystem object between folders and/or filesystem mounts. If the
    /// `Overwrite` flag is not set and the destfile entry already exists, the
    /// operation will fail.
    ///
    /// Use of the other flags (Native or Atomic) indicate a preference (but
    /// not a requirement) for the remote end to perform an atomic rename
    /// operation and/or using native system calls when possible.
    ///
    /// If no flags are specified then all flags are used.
    pub async fn rename(
        &self,
        src: &Path,
        dst: &Path,
        flags: Option<ssh2::RenameFlags>,
    ) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.rename(src, dst, flags) })
    }

    /// Remove a file on the remote filesystem
    pub async fn unlink(&self, file: &Path) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.unlink(file) })
    }

    /// Peel off the last error to happen on this SFTP instance.
    pub fn last_error(&self) -> Error {
        self.inner.last_error()
    }

    /// Translates a return code into a Rust-`Result`
    pub fn rc(&self, rc: c_int) -> Result<(), Error> {
        self.inner.rc(rc)
    }
}

impl<'sftp> File<'sftp> {
    pub(crate) fn new(file: ssh2::File<'sftp>, aio: Arc<Option<Aio>>) -> Self {
        Self { inner: file, aio }
    }

    /// Set the metadata for this handle.
    pub async fn setstat(&mut self, stat: FileStat) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.setstat(stat.clone()) })
    }

    /// Get the metadata for this handle.
    pub async fn stat(&mut self) -> Result<FileStat, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.stat() })
    }

    #[allow(missing_docs)]
    // sure wish I knew what this did...
    // TODO
    /*
    pub async fn statvfs(&mut self) -> Result<raw::LIBSSH2_SFTP_STATVFS, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.statvfs() })
    }
    */

    /// Reads a block of data from a handle and returns file entry information
    /// for the next entry, if any.
    ///
    /// Note that this provides raw access to the `readdir` function from
    /// libssh2. This will return an error when there are no more files to
    /// read, and files such as `.` and `..` will be included in the return
    /// values.
    ///
    /// Also note that the return paths will not be absolute paths, they are
    /// the filenames of the files in this directory.
    pub async fn readdir(&mut self) -> Result<(PathBuf, FileStat), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.readdir() })
    }

    /// This function causes the remote server to synchronize the file data and
    /// metadata to disk (like fsync(2)).
    ///
    /// For this to work requires fsync@openssh.com support on the server.
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
