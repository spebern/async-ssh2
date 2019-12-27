// TODO get rid of -37
#[macro_export]
macro_rules! into_the_future {
    ($poll_evented:ident; $cb:expr) => { into_the_future!('a; $poll_evented; $cb)};
    ($l:lifetime; $poll_evented:ident; $cb:expr) => {
        {
            struct ScopedFuture<$l, R, F: FnMut() -> Result<R, Error>> {
                cb: &$l mut F,
                poll_evented: Arc<Option<PollEvented<Io>>>,
            }

            impl<$l, R, F: FnMut() -> Result<R, Error>> Future for ScopedFuture<$l, R, F> {
                type Output = Result<R, Error>;

                fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                    let ready = Ready::readable();
                    loop {
                        let res = (&mut self.cb)();
                        match res {
                            Err(e) if e.code() == -37 => {
                                if let Some(ref poll_evented) = *self.poll_evented {
                                    // TODO write or read or both...
                                    poll_evented.clear_read_ready(cx, ready).unwrap();
                                }
                                return Poll::Pending;
                            },
                            Err(e) => {
                                return Poll::Ready(Err(e))
                            }
                            Ok(val) => return Poll::Ready(Ok(val)),
                        }
                    }
                }
            }

            let f = ScopedFuture {
                cb: $cb,
                poll_evented: $poll_evented,
            };

            f.await
        }
    };
}
