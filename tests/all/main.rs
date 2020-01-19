//#![deny(warnings)]

extern crate async_ssh2;
extern crate tempfile;

use std::{env, net::TcpStream};

mod agent;
mod channel;
mod knownhosts;
mod session;
mod sftp;

pub fn test_addr() -> String {
    let port = env::var("RUST_SSH2_FIXTURE_PORT")
        .map(|s| s.parse().unwrap())
        .unwrap_or(22);
    let addr = format!("127.0.0.1:{}", port);
    addr
}

pub fn socket() -> TcpStream {
    TcpStream::connect(&test_addr()).unwrap()
}

pub async fn authed_session() -> async_ssh2::Session {
    let user = env::var("USER").unwrap();
    let socket = socket();
    let mut sess = async_ssh2::Session::new().unwrap();
    sess.set_tcp_stream(socket).unwrap();
    sess.handshake().await.unwrap();
    assert!(!sess.authenticated());

    {
        let mut agent = sess.agent().unwrap();
        agent.connect().await.unwrap();
        agent.list_identities().unwrap();
        let identity = &agent.identities().unwrap()[0];
        agent.userauth(&user, &identity).await.unwrap();
    }
    assert!(sess.authenticated());
    sess
}
