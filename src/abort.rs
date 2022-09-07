use std::pin::Pin;

pub use futures::future::Aborted;
use futures::future::{select, Either, Map, Select};
use futures::{Future, FutureExt as FuturesExt};

pub type WithAbort<F, A> = Map<
    Select<F, A>,
    fn(<Select<F, A> as Future>::Output) -> Result<<F as Future>::Output, <A as Future>::Output>,
>;

pub trait FutureExt: Future {
    fn with_abort<F>(self, abort: F) -> WithAbort<Self, F>
    where
        F: Future + Unpin,
        Self: Sized + Unpin,
    {
        select(self, abort).map(|either| match either {
            Either::Left((r, _)) => Ok(r),
            Either::Right((err, _)) => Err(err),
        })
    }

    fn with_abort_unpin<F>(self, abort: F) -> WithAbort<Self, Pin<Box<F>>>
    where
        F: Future,
        Self: Sized + Unpin,
    {
        self.with_abort(Box::pin(abort))
    }

    fn with_unpin_abort_unpin<F>(self, abort: F) -> WithAbort<Pin<Box<Self>>, Pin<Box<F>>>
    where
        F: Future,
        Self: Sized,
    {
        Box::pin(self).with_abort_unpin(abort)
    }
}

impl<T> FutureExt for T where T: Future + ?Sized {}
