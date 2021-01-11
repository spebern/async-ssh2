use async_ssh2::Channel;
use futures::io::{AsyncReadExt, AsyncWriteExt};
use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
    thread,
};

/// Consume all available stdout and stderr data.
/// It is important to read both if you are using
/// channel.eof() to make assertions that the stream
/// is complete
async fn consume_stdio(channel: &mut Channel) -> (String, String) {
    let mut stdout = String::new();
    channel.read_to_string(&mut stdout).await.unwrap();

    let mut stderr = String::new();
    channel.stderr().read_to_string(&mut stderr).unwrap();

    eprintln!("stdout: {}", stdout);
    eprintln!("stderr: {}", stderr);

    (stdout, stderr)
}

#[tokio::test]
async fn smoke() {
    let sess = crate::authed_session().await;
    let mut channel = sess.channel_session().await.unwrap();

    fn must_be_send<T: Send>(_: &T) -> bool {
        true
    }
    assert!(must_be_send(&channel));
    assert!(must_be_send(&channel.stream(0)));

    channel.flush().await.unwrap();
    channel.exec("true").await.unwrap();
    consume_stdio(&mut channel).await;

    channel.wait_eof().await.unwrap();
    assert!(channel.eof());

    channel.close().await.unwrap();
    channel.wait_close().await.unwrap();
    assert_eq!(channel.exit_status().unwrap(), 0);
    assert!(channel.eof());
}

#[tokio::test]
async fn bad_smoke() {
    let sess = crate::authed_session().await;
    let mut channel = sess.channel_session().await.unwrap();
    channel.flush().await.unwrap();
    channel.exec("false").await.unwrap();
    consume_stdio(&mut channel).await;

    channel.wait_eof().await.unwrap();
    assert!(channel.eof());

    channel.close().await.unwrap();
    channel.wait_close().await.unwrap();
    assert_eq!(channel.exit_status().unwrap(), 1);
    assert!(channel.eof());
}

#[tokio::test]
async fn reading_data() {
    let sess = crate::authed_session().await;
    let mut channel = sess.channel_session().await.unwrap();
    channel.exec("echo foo").await.unwrap();

    let (output, _) = consume_stdio(&mut channel).await;
    assert_eq!(output, "foo\n");
}

#[tokio::test]
async fn handle_extended_data() {
    let sess = crate::authed_session().await;
    let mut channel = sess.channel_session().await.unwrap();
    channel
        .handle_extended_data(ssh2::ExtendedData::Merge)
        .await
        .unwrap();
    channel.exec("echo foo >&2").await.unwrap();
    let (output, _) = consume_stdio(&mut channel).await;
    // This is an ends_with test because stderr may have several
    // lines of misc output on travis macos hosts; it appears as
    // though the local shell configuration on travis macos is
    // broken and contributes to this :-/
    assert!(output.ends_with("foo\n"));
}

#[tokio::test]
async fn writing_data() {
    let sess = crate::authed_session().await;
    let mut channel = sess.channel_session().await.unwrap();
    channel.exec("read foo && echo $foo").await.unwrap();
    channel.write_all(b"foo\n").await.unwrap();

    let (output, _) = consume_stdio(&mut channel).await;
    assert_eq!(output, "foo\n");
}

#[tokio::test]
async fn eof() {
    let sess = crate::authed_session().await;
    let mut channel = sess.channel_session().await.unwrap();
    channel.adjust_receive_window(10, false).await.unwrap();
    channel.exec("read foo").await.unwrap();
    channel.send_eof().await.unwrap();
    let mut output = String::new();
    channel.read_to_string(&mut output).await.unwrap();
    assert_eq!(output, "");
}

#[tokio::test]
async fn shell() {
    let sess = crate::authed_session().await;
    let mut channel = sess.channel_session().await.unwrap();
    eprintln!("requesting pty");
    channel.request_pty("xterm", None, None).await.unwrap();
    eprintln!("shell");
    channel.shell().await.unwrap();
    eprintln!("close");
    channel.close().await.unwrap();
    eprintln!("done");
    consume_stdio(&mut channel).await;
}

#[tokio::test]
async fn setenv() {
    let sess = crate::authed_session().await;
    let mut channel = sess.channel_session().await.unwrap();
    let _ = channel.setenv("FOO", "BAR").await;
    channel.close().await.unwrap();
}

#[tokio::test]
async fn direct() {
    let a = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = a.local_addr().unwrap();
    let t = thread::spawn(move || {
        let mut s = a.accept().unwrap().0;
        let mut b = [0, 0, 0];
        s.read(&mut b).unwrap();
        assert_eq!(b, [1, 2, 3]);
        s.write_all(&[4, 5, 6]).unwrap();
    });
    let sess = crate::authed_session().await;
    let mut channel = sess
        .channel_direct_tcpip("127.0.0.1", addr.port(), None)
        .await
        .unwrap();
    channel.write_all(&[1, 2, 3]).await.unwrap();
    let mut r = [0, 0, 0];
    channel.read(&mut r).await.unwrap();
    assert_eq!(r, [4, 5, 6]);
    t.join().ok().unwrap();
}

#[tokio::test]
async fn forward() {
    let sess = crate::authed_session().await;
    let (mut listen, port) = sess
        .channel_forward_listen(39249, None, None)
        .await
        .unwrap();
    let t = thread::spawn(move || {
        let mut s = TcpStream::connect(&("127.0.0.1", port)).unwrap();
        let mut b = [0, 0, 0];
        s.read(&mut b).unwrap();
        assert_eq!(b, [1, 2, 3]);
        s.write_all(&[4, 5, 6]).unwrap();
    });

    let mut channel = listen.accept().await.unwrap();
    channel.write_all(&[1, 2, 3]).await.unwrap();
    let mut r = [0, 0, 0];
    channel.read(&mut r).await.unwrap();
    assert_eq!(r, [4, 5, 6]);
    t.join().ok().unwrap();
}

#[tokio::test]
async fn drop_nonblocking() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    let sess = crate::authed_session().await;

    thread::spawn(move || {
        let _s = listener.accept().unwrap();
    });

    let _ = sess
        .channel_direct_tcpip("127.0.0.1", addr.port(), None)
        .await;
    drop(sess);
}

#[tokio::test]
async fn nonblocking_before_exit_code() {
    let sess = crate::authed_session().await;
    let mut channel = sess.channel_session().await.unwrap();
    channel.send_eof().await.unwrap();
    let mut output = String::new();

    channel.exec("sleep 1; echo foo").await.unwrap();
    assert!(channel.read_to_string(&mut output).await.is_ok());

    channel.wait_eof().await.unwrap();
    channel.close().await.unwrap();
    channel.wait_close().await.unwrap();
    assert_eq!(output, "foo\n");
    assert!(channel.exit_status().unwrap() == 0);
}

#[tokio::test]
async fn exit_code_ignores_other_errors() {
    let sess = crate::authed_session().await;
    let mut channel = sess.channel_session().await.unwrap();
    channel.exec("true").await.unwrap();
    channel.wait_eof().await.unwrap();
    channel.close().await.unwrap();
    channel.wait_close().await.unwrap();
    let longdescription: String = ::std::iter::repeat('a').take(300).collect();
    assert!(sess.disconnect(None, &longdescription, None).await.is_err()); // max len == 256
    assert!(channel.exit_status().unwrap() == 0);
}

/*
#[test]
fn pty_modes_are_propagated() {
    let sess = ::authed_session();
    let mut channel = sess.channel_session().unwrap();
    eprintln!("requesting pty");

    let mut mode = ssh2::PtyModes::new();
    // intr is typically CTRL-C; setting it to unmodified `y`
    // should be very high signal that it took effect
    mode.set_character(ssh2::PtyModeOpcode::VINTR, Some('y'));

    channel.request_pty("xterm", Some(mode), None).unwrap();
    channel.exec("stty -a").unwrap();

    let (out, _err) = consume_stdio(&mut channel);
    channel.close().unwrap();

    // This may well be linux specific
    assert!(out.contains("intr = y"), "mode was propagated");
}
*/
