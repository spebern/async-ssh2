// TODO get rid of -37
#[macro_export]
macro_rules! into_the_future {
    ($aio:ident; $cb:expr) => {{
        struct ScopedFuture<'a, R, F: FnMut() -> Result<R, Error>> {
            cb: &'a mut F,
            aio: Arc<Option<Aio>>,
        }

        impl<'a, R, F: FnMut() -> Result<R, Error>> Future for ScopedFuture<'a, R, F> {
            type Output = Result<R, Error>;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                match (&mut self.cb)() {
                    Err(e) if e.code() == -37 => {
                        if let Some(ref aio) = *self.aio {
                            // TODO get rid of unwrap
                            aio.set_waker(cx).unwrap();
                        }
                        return Poll::Pending;
                    }
                    Err(e) => return Poll::Ready(Err(e)),
                    Ok(val) => return Poll::Ready(Ok(val)),
                }
            }
        }

        let f = ScopedFuture { cb: $cb, aio: $aio };

        f.await
    }};
}
