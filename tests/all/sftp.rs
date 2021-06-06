use futures::io::{AsyncReadExt, AsyncWriteExt};
use std::{
    fs::{self, File},
    io::prelude::*,
};
use tempfile::tempdir;
use tokio;

#[tokio::test]
async fn smoke() {
    let sess = crate::authed_session().await;
    sess.sftp().await.unwrap();
}

#[tokio::test]
async fn ops() {
    let td = tempdir().unwrap();
    File::create(&td.path().join("foo")).unwrap();
    fs::create_dir(&td.path().join("bar")).unwrap();

    let sess = crate::authed_session().await;
    let sftp = sess.sftp().await.unwrap();
    sftp.opendir(&td.path().join("bar")).await.unwrap();
    let mut foo = sftp.open(&td.path().join("foo")).await.unwrap();
    sftp.mkdir(&td.path().join("bar2"), 0o755).await.unwrap();
    assert!(fs::metadata(&td.path().join("bar2"))
        .map(|m| m.is_dir())
        .unwrap_or(false));
    sftp.rmdir(&td.path().join("bar2")).await.unwrap();

    sftp.create(&td.path().join("foo5"))
        .await
        .unwrap()
        .write_all(b"foo")
        .await
        .unwrap();
    let mut v = Vec::new();
    File::open(&td.path().join("foo5"))
        .unwrap()
        .read_to_end(&mut v)
        .unwrap();
    assert_eq!(v, b"foo");

    assert_eq!(
        sftp.stat(&td.path().join("foo")).await.unwrap().size,
        Some(0)
    );
    v.truncate(0);
    foo.read_to_end(&mut v).await.unwrap();
    assert_eq!(v, Vec::new());

    foo.close().await.unwrap();

    sftp.symlink(&td.path().join("foo"), &td.path().join("foo2"))
        .await
        .unwrap();
    let readlink = sftp.readlink(&td.path().join("foo2")).await.unwrap();
    assert!(readlink == td.path().join("foo"));
    let realpath = sftp.realpath(&td.path().join("foo2")).await.unwrap();
    assert_eq!(realpath, td.path().join("foo").canonicalize().unwrap());

    let files = sftp.readdir(&td.path()).await.unwrap();
    assert_eq!(files.len(), 4);

    // This test fails, see FIXME in the implementation
    //sftp.shutdown().await.unwrap();
}
