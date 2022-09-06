use std::error::Error;
use std::fmt::{Debug, Display};
use std::task::Poll;

use futures::{Future, FutureExt};

#[derive(Clone)]
pub struct CancelToken(async_broadcast::Receiver<()>);

impl CancelToken {
    pub fn new() -> (Self, impl Drop) {
        let (tx, rx) = async_broadcast::broadcast(1);
        (Self(rx), tx)
    }
}

impl Future for CancelToken {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        self.0.recv().poll_unpin(cx).map(|_| ())
    }
}

pub trait CancelFutureExt: Future {
    fn with_cancel(self, cancel: CancelToken) -> WithCancel<Self>
    where
        Self: Sized,
    {
        WithCancel {
            inner: Some((self, cancel)),
        }
    }
}

impl<T> CancelFutureExt for T where T: Future + ?Sized {}

pub struct WithCancel<F: Future> {
    inner: Option<(F, CancelToken)>,
}

pub struct Cancelled;

impl Debug for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cancelled")
    }
}

impl Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(&self, f)
    }
}

impl Error for Cancelled {}

impl<F> Future for WithCancel<F>
where
    F: Future + Unpin,
{
    type Output = Result<<F as Future>::Output, Cancelled>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let (mut f, mut cancel) = self.inner.take().expect("cannot call WithCancel twice");

        if let Poll::Ready(_) = cancel.poll_unpin(cx) {
            return Poll::Ready(Err(Cancelled));
        }

        if let Poll::Ready(r) = f.poll_unpin(cx) {
            return Poll::Ready(Ok(r));
        }

        self.inner = Some((f, cancel));
        Poll::Pending
    }
}
