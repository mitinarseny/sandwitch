use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::future::LocalFutureObj;
use futures::task::LocalSpawn;
use futures::{
    future::{Future, FutureObj},
    stream::{
        FusedStream, FuturesOrdered as StdFuturesOrdered, FuturesUnordered as StdFuturesUnordered,
        Stream,
    },
    task::{Spawn, SpawnError},
};
use metrics::{Counter, Gauge};
use pin_project::pin_project;

#[pin_project]
pub struct InFlight<St>
where
    St: Stream,
{
    #[pin]
    inner: St,
    in_flight: Gauge,
}

impl<St> Deref for InFlight<St>
where
    St: Stream,
{
    type Target = St;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<St> DerefMut for InFlight<St>
where
    St: Stream,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<St> InFlight<St>
where
    St: Stream,
{
    fn on_push(&self) {
        self.in_flight.increment(1.0);
    }

    fn on_pop(&self) {
        self.in_flight.decrement(1.0);
    }

    fn on_clear(&self) {
        self.in_flight.set(0.0);
    }
}

impl<St> Stream for InFlight<St>
where
    St: Stream,
{
    type Item = <St as Stream>::Item;

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

impl<St> FusedStream for InFlight<St>
where
    St: FusedStream,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

pub type FuturesOrdered<Fut> = InFlight<StdFuturesOrdered<Fut>>;

impl<Fut> FuturesOrdered<Fut>
where
    Fut: Future,
{
    pub fn new(in_flight: Gauge) -> Self {
        Self {
            inner: StdFuturesOrdered::new(),
            in_flight,
        }
    }

    pub fn push_back(&mut self, f: Fut) {
        self.on_push();
        self.inner.push_back(f)
    }

    pub fn push_front(&mut self, f: Fut) {
        self.on_push();
        self.inner.push_front(f)
    }
}

pub type FuturesUnordered<Fut> = InFlight<StdFuturesUnordered<Fut>>;

impl<Fut> FuturesUnordered<Fut>
where
    Fut: Future,
{
    pub fn new(in_flight: Gauge) -> Self {
        Self {
            inner: StdFuturesUnordered::new(),
            in_flight,
        }
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

impl<St, Fut> Extend<Fut> for InFlight<St>
where
    Fut: Future,
    St: Stream + Extend<Fut>,
{
    fn extend<T: IntoIterator<Item = Fut>>(&mut self, iter: T) {
        self.inner
            .extend(iter.into_iter().inspect(|_| self.in_flight.increment(1.0)))
    }
}

impl<St> Spawn for InFlight<St>
where
    St: Stream + Spawn,
{
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

impl<St> LocalSpawn for InFlight<St>
where
    St: Stream + LocalSpawn,
{
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
