macro_rules! ready {
    ($e:expr $(,)?) => {
        match $e {
            std::task::Poll::Ready(t) => t,
            std::task::Poll::Pending => return std::task::Poll::Pending,
        }
    };
}

macro_rules! into_the_future {
    ($aio:ident; $cb:expr) => {{
        struct ScopedFuture<'a, R, F: FnMut() -> Result<R, ssh2::Error>> {
            cb: &'a mut F,
            aio: Arc<Option<Aio>>,
        }

        impl<'a, R, F: FnMut() -> Result<R, ssh2::Error>> Future for ScopedFuture<'a, R, F> {
            type Output = Result<R, Error>;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                loop {
                    match (&mut self.cb)() {
                        Err(e)
                            if io::Error::from(ssh2::Error::from_errno(e.code())).kind()
                                == io::ErrorKind::WouldBlock =>
                        {
                            if let Some(ref aio) = *self.aio {
                                ready!(aio.poll(cx).map_err(Error::from))?;
                            } else {
                                continue;
                            }
                        }
                        Err(e) => return Poll::Ready(Err(Error::from(e))),
                        Ok(val) => return Poll::Ready(Ok(val)),
                    }
                }
            }
        }

        let f = ScopedFuture { cb: $cb, aio: $aio };

        f.await
    }};
}
