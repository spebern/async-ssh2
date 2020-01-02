use crate::{
    agent::Agent, aio::Aio, channel::Channel, into_the_future, listener::Listener, sftp::Sftp,
    Error,
};
#[cfg(unix)]
use libc::c_int;

use ssh2::{
    self, DisconnectCode, HashType, HostKeyType, KeyboardInteractivePrompt, KnownHosts, MethodType,
    ScpFileStat,
};
use std::{
    cell::Ref,
    convert::From,
    future::Future,
    io,
    net::TcpStream,
    path::Path,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// See [`Session`](ssh2::Session).
pub struct Session {
    inner: ssh2::Session,
    aio: Arc<Option<Aio>>,
}

// The compiler doesn't know that it is Send safe because of the raw
// pointer inside.  We know that the way that it is used by libssh2
// and this crate is Send safe.
unsafe impl Send for Session {}

impl Session {
    /// See [`new`](ssh2::Session::new).
    pub fn new() -> Result<Session, Error> {
        let session = ssh2::Session::new()?;
        session.set_blocking(false);
        Ok(Self {
            inner: session,
            aio: Arc::new(None),
        })
    }

    /// See [`set_banner`](ssh2::Session::set_banner).
    pub async fn set_banner(&self, banner: &str) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.set_banner(banner) })
    }

    /// See [`set_allow_sigpipe`](ssh2::Session::set_allow_sigpipe).
    pub fn set_allow_sigpipe(&self, block: bool) {
        self.inner.set_allow_sigpipe(block)
    }

    /// See [`set_allow_sigpipe`](ssh2::Session::set_compress).
    pub fn set_compress(&self, compress: bool) {
        self.inner.set_compress(compress)
    }

    /// See [`is_blocking`](ssh2::Session::is_blocking).
    pub fn is_blocking(&self) -> bool {
        self.inner.is_blocking()
    }

    /// See [`set_timeout`](ssh2::Session::set_timeout).
    pub fn set_timeout(&self, timeout_ms: u32) {
        self.inner.set_timeout(timeout_ms)
    }

    /// See [`timeout`](ssh2::Session::timeout).
    pub fn timeout(&self) -> u32 {
        self.inner.timeout()
    }

    /// See [`handshake`](ssh2::Session::handshake).
    pub async fn handshake(&mut self) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.handshake() })
    }

    /// See [`set_tcp_stream`](ssh2::Session::set_tcp_stream).
    pub fn set_tcp_stream(&mut self, stream: TcpStream) -> Result<(), Error> {
        let aio = Aio::new(stream.try_clone()?, self.inner.clone())?;
        self.aio = Arc::new(Some(aio));
        self.inner.set_tcp_stream(stream);
        Ok(())
    }

    /// See [`tcp_stream`](ssh2::Session::tcp_stream).
    pub fn tcp_stream(&self) -> Ref<Option<TcpStream>> {
        self.inner.tcp_stream()
    }

    /// See [`userauth_password`](ssh2::Session::userauth_password).
    pub async fn userauth_password(&self, username: &str, password: &str) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.userauth_password(username, password) })
    }

    /// See [`userauth_keyboard_interactive`](ssh2::Session::userauth_keyboard_interactive).
    pub fn userauth_keyboard_interactive<P: KeyboardInteractivePrompt>(
        &self,
        _username: &str,
        _prompter: &mut P,
    ) -> Result<(), Error> {
        todo!();
    }

    /// See [`userauth_agent`](ssh2::Session::userauth_agent).
    pub async fn userauth_agent(&self, username: &str) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.userauth_agent(username) })
    }

    /// See [`userauth_pubkey_file`](ssh2::Session::userauth_pubkey_file).
    pub async fn userauth_pubkey_file(
        &self,
        username: &str,
        pubkey: Option<&Path>,
        privatekey: &Path,
        passphrase: Option<&str>,
    ) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.userauth_pubkey_file(username, pubkey, privatekey, passphrase) })
    }

    /// See [`userauth_pubkey_memory`](ssh2::Session::userauth_pubkey_memory).
    #[cfg(unix)]
    pub async fn userauth_pubkey_memory(
        &self,
        username: &str,
        pubkeydata: Option<&str>,
        privatekeydata: &str,
        passphrase: Option<&str>,
    ) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.userauth_pubkey_memory(username, pubkeydata, privatekeydata, passphrase) })
    }

    /// See [`userauth_hostbased_file`](ssh2::Session::userauth_hostbased_file).
    #[allow(missing_docs)]
    pub async fn userauth_hostbased_file(
        &self,
        username: &str,
        publickey: &Path,
        privatekey: &Path,
        passphrase: Option<&str>,
        hostname: &str,
        local_username: Option<&str>,
    ) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.userauth_hostbased_file(username, publickey, privatekey, passphrase, hostname, local_username) })
    }

    /// See [`authenticated`](ssh2::Session::authenticated).
    pub fn authenticated(&self) -> bool {
        self.inner.authenticated()
    }

    /// See [`auth_methods`](ssh2::Session::auth_methods).
    pub async fn auth_methods(&self, username: &str) -> Result<&str, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.auth_methods(username) })
    }

    /// See [`method_pref`](ssh2::Session::method_pref).
    pub async fn method_pref(&self, method_type: MethodType, prefs: &str) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.method_pref(method_type, prefs) })
    }

    /// See [`methods`](ssh2::Session::methods).
    pub fn methods(&self, method_type: MethodType) -> Option<&str> {
        self.inner.methods(method_type)
    }

    /// See [`supported_algs`](ssh2::Session::supported_algs).
    pub fn supported_algs(&self, method_type: MethodType) -> Result<Vec<&'static str>, Error> {
        self.inner.supported_algs(method_type).map_err(From::from)
    }

    /// See [`agent`](ssh2::Session::agent).
    pub fn agent(&self) -> Result<Agent, Error> {
        let agent = self.inner.agent()?;
        Ok(Agent::new(agent, self.aio.clone()))
    }

    /// See [`known_hosts`](ssh2::Session::known_hosts).
    pub fn known_hosts(&self) -> Result<KnownHosts, Error> {
        self.inner.known_hosts().map_err(From::from)
    }

    /// See [`channel_session`](ssh2::Session::channel_session).
    pub async fn channel_session(&self) -> Result<Channel, Error> {
        let aio = self.aio.clone();
        let channel = into_the_future!(aio; &mut || { self.inner.channel_session() })?;
        Ok(Channel::new(channel, self.aio.clone()))
    }

    /// See [`channel_direct_tcpip`](ssh2::Session::channel_direct_tcpip).
    pub async fn channel_direct_tcpip(
        &self,
        host: &str,
        port: u16,
        src: Option<(&str, u16)>,
    ) -> Result<Channel, Error> {
        let aio = self.aio.clone();
        let channel =
            into_the_future!(aio; &mut || { self.inner.channel_direct_tcpip(host, port, src) })?;
        Ok(Channel::new(channel, self.aio.clone()))
    }

    /// See [`channel_forward_listen`](ssh2::Session::channel_forward_listen).
    pub async fn channel_forward_listen(
        &self,
        remote_port: u16,
        host: Option<&str>,
        queue_maxsize: Option<u32>,
    ) -> Result<(Listener, u16), Error> {
        let aio = self.aio.clone();
        let (listener, port) = into_the_future!(aio; &mut || { self.inner.channel_forward_listen(remote_port, host, queue_maxsize) })?;
        Ok((Listener::new(listener, self.aio.clone()), port))
    }

    /// See [`scp_recv`](ssh2::Session::scp_recv).
    pub async fn scp_recv(&self, path: &Path) -> Result<(Channel, ScpFileStat), Error> {
        let aio = self.aio.clone();
        let (channel, file_stat) = into_the_future!(aio; &mut || { self.inner.scp_recv(path) })?;
        Ok((Channel::new(channel, self.aio.clone()), file_stat))
    }

    /// See [`scp_send`](ssh2::Session::scp_send).
    pub async fn scp_send(
        &self,
        remote_path: &Path,
        mode: i32,
        size: u64,
        times: Option<(u64, u64)>,
    ) -> Result<Channel, Error> {
        let aio = self.aio.clone();
        let channel =
            into_the_future!(aio; &mut || { self.inner.scp_send(remote_path, mode, size, times) })?;
        Ok(Channel::new(channel, self.aio.clone()))
    }

    /// See [`sftp`](ssh2::Session::sftp).
    pub async fn sftp(&self) -> Result<Sftp, Error> {
        let aio = self.aio.clone();
        let sftp = into_the_future!(aio; &mut || { self.inner.sftp() })?;
        Ok(Sftp::new(sftp, self.aio.clone()))
    }

    /// See [`channel_open`](ssh2::Session::channel_open).
    pub async fn channel_open(
        &self,
        channel_type: &str,
        window_size: u32,
        packet_size: u32,
        message: Option<&str>,
    ) -> Result<Channel, Error> {
        let aio = self.aio.clone();
        let channel = into_the_future!(aio; &mut || { self.inner.channel_open(channel_type, window_size, packet_size, message) })?;
        Ok(Channel::new(channel, self.aio.clone()))
    }

    /// See [`banner`](ssh2::Session::banner).
    pub fn banner(&self) -> Option<&str> {
        self.inner.banner()
    }

    /// See [`banner_bytes`](ssh2::Session::banner_bytes).
    pub fn banner_bytes(&self) -> Option<&[u8]> {
        self.inner.banner_bytes()
    }

    /// See [`host_key`](ssh2::Session::host_key).
    pub fn host_key(&self) -> Option<(&[u8], HostKeyType)> {
        self.inner.host_key()
    }

    /// See [`host_key_hash`](ssh2::Session::host_key_hash).
    pub fn host_key_hash(&self, hash: HashType) -> Option<&[u8]> {
        self.inner.host_key_hash(hash)
    }

    /// See [`set_keepalive`](ssh2::Session::set_keepalive).
    pub fn set_keepalive(&self, want_reply: bool, interval: u32) {
        self.inner.set_keepalive(want_reply, interval)
    }

    /// See [`keepalive_send`](ssh2::Session::keepalive_send).
    pub async fn keepalive_send(&self) -> Result<u32, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.keepalive_send() })
    }

    /// See [`disconnect`](ssh2::Session::disconnect).
    pub async fn disconnect(
        &self,
        reason: Option<DisconnectCode>,
        description: &str,
        lang: Option<&str>,
    ) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.disconnect(reason, description, lang) })
    }

    /// See [`rc`](ssh2::Session::rc).
    pub fn rc(&self, rc: c_int) -> Result<(), Error> {
        self.inner.rc(rc).map_err(From::from)
    }
}
