use crate::{
    agent::Agent, aio::Aio, channel::Channel, into_the_future, listener::Listener, sftp::Sftp,
};
#[cfg(unix)]
use libc::c_int;

use ssh2::{
    self, DisconnectCode, Error, HashType, HostKeyType, KeyboardInteractivePrompt, KnownHosts,
    MethodType, ScpFileStat,
};
use std::{
    cell::Ref,
    future::Future,
    net::TcpStream,
    os::unix::io::AsRawFd,
    path::Path,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// An SSH session, typically representing one TCP connection.
///
/// All other structures are based on an SSH session and cannot outlive a
/// session. Sessions are created and then have the TCP socket handed to them
/// (via the `set_tcp_stream` method).
pub struct Session {
    inner: ssh2::Session,
    aio: Arc<Option<Aio>>,
}

impl Session {
    /// Initializes an SSH session object.
    ///
    /// This function does not associate the session with a remote connection
    /// just yet. Various configuration options can be set such as the blocking
    /// mode, compression, sigpipe, the banner, etc. To associate this session
    /// with a TCP connection, use the `set_tcp_stream` method pass in an
    /// already-established TCP socket, and then follow up with a call to
    /// `handshake` to perform the ssh protocol handshake.
    pub fn new() -> Result<Session, Error> {
        let session = ssh2::Session::new()?;
        session.set_blocking(false);
        Ok(Self {
            inner: session,
            aio: Arc::new(None),
        })
    }

    /// Set the SSH protocol banner for the local client
    ///
    /// Set the banner that will be sent to the remote host when the SSH session
    /// is started with handshake(). This is optional; a banner
    /// corresponding to the protocol and libssh2 version will be sent by
    /// default.
    pub async fn set_banner(&self, banner: &str) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.set_banner(banner) })
    }

    /// Flag indicating whether SIGPIPE signals will be allowed or blocked.
    ///
    /// By default (on relevant platforms) this library will attempt to block
    /// and catch SIGPIPE signals. Setting this flag to `true` will cause
    /// the library to not attempt to block SIGPIPE from the underlying socket
    /// layer.
    pub fn set_allow_sigpipe(&self, block: bool) {
        self.inner.set_allow_sigpipe(block)
    }

    /// Flag indicating whether this library will attempt to negotiate
    /// compression.
    ///
    /// If set - before the connection negotiation is performed - libssh2 will
    /// try to negotiate compression enabling for this connection. By default
    /// libssh2 will not attempt to use compression.
    pub fn set_compress(&self, compress: bool) {
        self.inner.set_compress(compress)
    }

    /// Returns whether the session was previously set to nonblocking.
    pub fn is_blocking(&self) -> bool {
        self.inner.is_blocking()
    }

    /// Set timeout for blocking functions.
    ///
    /// Set the timeout in milliseconds for how long a blocking the libssh2
    /// function calls may wait until they consider the situation an error and
    /// return an error.
    ///
    /// By default or if you set the timeout to zero, libssh2 has no timeout
    /// for blocking functions.
    pub fn set_timeout(&self, timeout_ms: u32) {
        self.inner.set_timeout(timeout_ms)
    }

    /// Returns the timeout, in milliseconds, for how long blocking calls may
    /// wait until they time out.
    ///
    /// A timeout of 0 signifies no timeout.
    pub fn timeout(&self) -> u32 {
        self.inner.timeout()
    }

    /// Begin transport layer protocol negotiation with the connected host.
    ///
    /// You must call this after associating the session with a tcp stream
    /// via the `set_tcp_stream` function.
    pub async fn handshake(&mut self) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.handshake() })
    }

    /// The session takes ownership of the socket provided.
    /// You may use the tcp_stream() method to obtain a reference
    /// to it later.
    ///
    /// It is also highly recommended that the stream provided is not used
    /// concurrently elsewhere for the duration of this session as it may
    /// interfere with the protocol.
    pub fn set_tcp_stream(&mut self, stream: TcpStream) {
        let aio = Aio::new(stream.as_raw_fd(), self.inner.clone());
        self.aio = Arc::new(Some(aio));
        self.inner.set_tcp_stream(stream);
    }

    /// Returns a reference to the stream that was associated with the Session
    /// by the Session::handshake method.
    pub fn tcp_stream(&self) -> Ref<Option<TcpStream>> {
        self.inner.tcp_stream()
    }

    /// Attempt basic password authentication.
    ///
    /// Note that many SSH servers which appear to support ordinary password
    /// authentication actually have it disabled and use Keyboard Interactive
    /// authentication (routed via PAM or another authentication backed)
    /// instead.
    pub async fn userauth_password(&self, username: &str, password: &str) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.userauth_password(username, password) })
    }

    /// Attempt keyboard interactive authentication.
    ///
    /// You must supply a callback function to
    pub fn userauth_keyboard_interactive<P: KeyboardInteractivePrompt>(
        &self,
        _username: &str,
        _prompter: &mut P,
    ) -> Result<(), Error> {
        todo!();
    }

    /// Attempt to perform SSH agent authentication.
    ///
    /// This is a helper method for attempting to authenticate the current
    /// connection with the first public key found in an SSH agent. If more
    /// control is needed than this method offers, it is recommended to use
    /// `agent` directly to control how the identity is found.
    pub async fn userauth_agent(&self, username: &str) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.userauth_agent(username) })
    }

    /// Attempt public key authentication using a PEM encoded private key file
    /// stored on disk.
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

    /// Attempt public key authentication using a PEM encoded private key from
    /// memory. Public key is computed from private key if none passed.
    /// This is available only for `unix` targets, as it relies on openssl.
    /// It is therefore recommended to use `#[cfg(unix)]` or otherwise test for
    /// the `unix` compliation target when using this function.
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

    // Umm... I wish this were documented in libssh2?
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

    /// Indicates whether or not the named session has been successfully
    /// authenticated.
    pub fn authenticated(&self) -> bool {
        self.inner.authenticated()
    }

    /// Send a SSH_USERAUTH_NONE request to the remote host.
    ///
    /// Unless the remote host is configured to accept none as a viable
    /// authentication scheme (unlikely), it will return SSH_USERAUTH_FAILURE
    /// along with a listing of what authentication schemes it does support. In
    /// the unlikely event that none authentication succeeds, this method with
    /// return an error. This case may be distinguished from a failing case by
    /// examining the return value of the `authenticated` method.
    ///
    /// The return value is a comma-separated string of supported auth schemes,
    /// and may be an empty string.
    pub async fn auth_methods(&self, username: &str) -> Result<&str, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.auth_methods(username) })
    }

    /// Set preferred key exchange method
    ///
    /// The preferences provided are a comma delimited list of preferred methods
    /// to use with the most preferred listed first and the least preferred
    /// listed last. If a method is listed which is not supported by libssh2 it
    /// will be ignored and not sent to the remote host during protocol
    /// negotiation.
    pub async fn method_pref(&self, method_type: MethodType, prefs: &str) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.method_pref(method_type, prefs) })
    }

    /// Return the currently active algorithms.
    ///
    /// Returns the actual method negotiated for a particular transport
    /// parameter. May return `None` if the session has not yet been started.
    pub fn methods(&self, method_type: MethodType) -> Option<&str> {
        self.inner.methods(method_type)
    }

    /// Get list of supported algorithms.
    pub fn supported_algs(&self, method_type: MethodType) -> Result<Vec<&'static str>, Error> {
        self.inner.supported_algs(method_type)
    }

    /// Init an ssh-agent handle.
    ///
    /// The returned agent will still need to be connected manually before use.
    pub fn agent(&self) -> Result<Agent, Error> {
        let agent = self.inner.agent()?;
        Ok(Agent::new(agent, self.aio.clone()))
    }

    /// Init a collection of known hosts for this session.
    ///
    /// Returns the handle to an internal representation of a known host
    /// collection.
    pub fn known_hosts(&self) -> Result<KnownHosts, Error> {
        self.inner.known_hosts()
    }

    /// Establish a new session-based channel.
    ///
    /// This method is commonly used to create a channel to execute commands
    /// over or create a new login shell.
    pub async fn channel_session(&self) -> Result<Channel, Error> {
        let aio = self.aio.clone();
        let channel = into_the_future!(aio; &mut || { self.inner.channel_session() })?;
        Ok(Channel::new(channel, self.aio.clone()))
    }

    /// Tunnel a TCP connection through an SSH session.
    ///
    /// Tunnel a TCP/IP connection through the SSH transport via the remote host
    /// to a third party. Communication from the client to the SSH server
    /// remains encrypted, communication from the server to the 3rd party host
    /// travels in cleartext.
    ///
    /// The optional `src` argument is the host/port to tell the SSH server
    /// where the connection originated from.
    ///
    /// The `Channel` returned represents a connection between this host and the
    /// specified remote host.
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

    /// Instruct the remote SSH server to begin listening for inbound TCP/IP
    /// connections.
    ///
    /// New connections will be queued by the library until accepted by the
    /// `accept` method on the returned `Listener`.
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

    /// Request a file from the remote host via SCP.
    ///
    /// The path specified is a path on the remote host which will attempt to be
    /// sent over the returned channel. Some stat information is also returned
    /// about the remote file to prepare for receiving the file.
    pub async fn scp_recv(&self, path: &Path) -> Result<(Channel, ScpFileStat), Error> {
        let aio = self.aio.clone();
        let (channel, file_stat) = into_the_future!(aio; &mut || { self.inner.scp_recv(path) })?;
        Ok((Channel::new(channel, self.aio.clone()), file_stat))
    }

    /// Send a file to the remote host via SCP.
    ///
    /// The `remote_path` provided will the remote file name. The `times`
    /// argument is a tuple of (mtime, atime), and will default to the remote
    /// host's current time if not specified.
    ///
    /// The size of the file, `size`, must be known ahead of time before
    /// transmission.
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

    /// Open a channel and initialize the SFTP subsystem.
    ///
    /// Although the SFTP subsystem operates over the same type of channel as
    /// those exported by the Channel API, the protocol itself implements its
    /// own unique binary packet protocol which must be managed with the
    /// methods on `Sftp`.
    pub async fn sftp(&self) -> Result<Sftp, Error> {
        let aio = self.aio.clone();
        let sftp = into_the_future!(aio; &mut || { self.inner.sftp() })?;
        Ok(Sftp::new(sftp, self.aio.clone()))
    }

    /// Allocate a new channel for exchanging data with the server.
    ///
    /// This is typically not called directly but rather through
    /// `channel_session`, `channel_direct_tcpip`, or `channel_forward_listen`.
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

    /// Get the remote banner
    ///
    /// Once the session has been setup and handshake() has completed
    /// successfully, this function can be used to get the server id from the
    /// banner each server presents.
    ///
    /// May return `None` on invalid utf-8 or if an error has ocurred.
    pub fn banner(&self) -> Option<&str> {
        self.inner.banner()
    }

    /// See `banner`.
    ///
    /// Will only return `None` if an error has ocurred.
    pub fn banner_bytes(&self) -> Option<&[u8]> {
        self.inner.banner_bytes()
    }

    /// Get the remote key.
    ///
    /// Returns `None` if something went wrong.
    pub fn host_key(&self) -> Option<(&[u8], HostKeyType)> {
        self.inner.host_key()
    }

    /// Returns the computed digest of the remote system's hostkey.
    ///
    /// The bytes returned are the raw hash, and are not printable. If the hash
    /// is not yet available `None` is returned.
    pub fn host_key_hash(&self, hash: HashType) -> Option<&[u8]> {
        self.inner.host_key_hash(hash)
    }

    /// Set how often keepalive messages should be sent.
    ///
    /// The want_reply argument indicates whether the keepalive messages should
    /// request a response from the server.
    ///
    /// The interval argument is number of seconds that can pass without any
    /// I/O, use 0 (the default) to disable keepalives. To avoid some busy-loop
    /// corner-cases, if you specify an interval of 1 it will be treated as 2.
    pub fn set_keepalive(&self, want_reply: bool, interval: u32) {
        self.inner.set_keepalive(want_reply, interval)
    }

    /// Send a keepalive message if needed.
    ///
    /// Returns how many seconds you can sleep after this call before you need
    /// to call it again.
    pub async fn keepalive_send(&self) -> Result<u32, Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.keepalive_send() })
    }

    /// Terminate the transport layer.
    ///
    /// Send a disconnect message to the remote host associated with session,
    /// along with a reason symbol and a verbose description.
    ///
    /// Note that this does *not* close the underlying socket.
    pub async fn disconnect(
        &self,
        reason: Option<DisconnectCode>,
        description: &str,
        lang: Option<&str>,
    ) -> Result<(), Error> {
        let aio = self.aio.clone();
        into_the_future!(aio; &mut || { self.inner.disconnect(reason, description, lang) })
    }

    /// Translate a return code into a Rust-`Result`.
    pub fn rc(&self, rc: c_int) -> Result<(), Error> {
        self.inner.rc(rc)
    }
}
