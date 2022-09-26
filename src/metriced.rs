use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::future::LocalFutureObj;
use futures::task::LocalSpawn;
use futures::{
    future::{Future, FutureObj},
    stream::{FusedStream, FuturesUnordered as StdFuturesUnordered, Stream},
    task::{Spawn, SpawnError},
};
use metrics::{Counter, Gauge};
use pin_project::pin_project;

#[pin_project]
pub struct FuturesUnordered<Fut> {
    #[pin]
    inner: StdFuturesUnordered<Fut>,
    in_flight: Gauge,
}

impl<Fut> Deref for FuturesUnordered<Fut> {
    type Target = StdFuturesUnordered<Fut>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<Fut> DerefMut for FuturesUnordered<Fut> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<Fut> FuturesUnordered<Fut> {
    pub fn new(in_flight: Gauge) -> Self {
        Self {
            inner: StdFuturesUnordered::new(),
            in_flight,
        }
    }

    fn on_push(&self) {
        self.in_flight.increment(1.0);
    }

    fn on_pop(&self) {
        self.in_flight.decrement(1.0);
    }

    fn on_clear(&self) {
        self.in_flight.set(0.0);
    }

    pub fn push(&self, f: Fut) {
        self.on_push();
        self.inner.push(f)
    }

    pub fn clear(&mut self) {
        self.on_clear();
        self.inner.clear()
    }
}

impl<Fut> Stream for FuturesUnordered<Fut>
where
    Fut: Future,
{
    type Item = <StdFuturesUnordered<Fut> as Stream>::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().project();
        let r = this.inner.poll_next(cx);
        if matches!(r, Poll::Ready(Some(_))) {
            self.on_pop();
        }
        r
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<Fut> FusedStream for FuturesUnordered<Fut>
where
    Fut: Future,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

impl<Fut> Extend<Fut> for FuturesUnordered<Fut>
where
    Fut: Future,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Fut>,
    {
        for item in iter {
            self.push(item);
        }
    }
}

impl Spawn for FuturesUnordered<FutureObj<'_, ()>> {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        let r = self.inner.spawn_obj(future);
        if r.is_ok() {
            self.on_push();
        }
        r
    }

    fn status(&self) -> Result<(), SpawnError> {
        self.inner.status()
    }
}

impl LocalSpawn for FuturesUnordered<LocalFutureObj<'_, ()>> {
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        let r = self.inner.spawn_local_obj(future);
        if r.is_ok() {
            self.on_push();
        }
        r
    }

    fn status_local(&self) -> Result<(), SpawnError> {
        self.inner.status_local()
    }
}

#[pin_project]
pub struct Counted<St>
where
    St: Stream,
{
    #[pin]
    inner: St,
    counter: Counter,
}

impl<St> Stream for Counted<St>
where
    St: Stream,
{
    type Item = St::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let r = this.inner.poll_next(cx);
        if matches!(r, Poll::Ready(Some(_))) {
            this.counter.increment(1);
        }
        r
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<St> FusedStream for Counted<St>
where
    St: FusedStream,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

pub trait StreamExt: Stream + Sized {
    fn counted(self, counter: Counter) -> Counted<Self> {
        Counted {
            inner: self,
            counter,
        }
    }
}

impl<St> StreamExt for St where St: Stream {}
