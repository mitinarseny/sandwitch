use std::ops::Deref;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{Future, Stream, TryFuture, TryStream};
use pin_project::pin_project;
use tokio::time::{Duration, Instant};

use super::with::With;

#[derive(Clone, Copy, Debug)]
pub struct Timed<T>(With<T, Instant>);

impl<T> Deref for Timed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(dead_code)]
impl<T> Timed<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self::with(value, Instant::now())
    }

    #[inline]
    pub fn with(value: T, at: Instant) -> Self {
        Self(With::new_with(value, at))
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }

    #[inline]
    pub fn at(&self) -> Instant {
        *AsRef::<Instant>::as_ref(&self.0)
    }

    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.at().elapsed()
    }


    #[inline]
    pub fn update(self, at: Instant) -> Self {
        Self::with(self.0.into_inner(), at)
    }

    #[inline]
    pub fn update_now(self) -> Self {
        self.update(Instant::now())
    }

    #[inline]
    pub fn set<U>(self, value: U) -> Timed<U> {
        Timed(self.0.set(value))
    }

    #[inline]
    pub fn map<U, F>(self, f: F) -> Timed<U>
    where
        F: FnOnce(T) -> U,
    {
        Timed(self.0.map(f))
    }
}

impl<T> From<T> for Timed<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Timed<Option<T>> {
    pub fn transpose(self) -> Option<Timed<T>> {
        let at = self.at();
        self.0.into_inner().map(|v| Timed::with(v, at))
    }
}

impl<T, E> Timed<Result<T, E>> {
    pub fn transpose(self) -> Result<Timed<T>, E> {
        let at = self.at();
        self.0.into_inner().map(|v| Timed::with(v, at))
    }
}

#[pin_project]
#[must_use = "futures do nothing unless polled"]
pub struct TimedFuture<Fut>
where
    Fut: Future,
{
    #[pin]
    inner: Fut,
    start: Option<Instant>,
}

impl<Fut> Future for TimedFuture<Fut>
where
    Fut: Future,
{
    type Output = Timed<Fut::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let start = this.start.get_or_insert_with(Instant::now);

        this.inner.poll(cx).map(move |v| Timed::with(v, *start))
    }
}

pub trait FutureExt: Future + Sized {
    fn timed(self) -> TimedFuture<Self> {
        TimedFuture {
            inner: self,
            start: None,
        }
    }
}

impl<F> FutureExt for F where F: Future {}

#[pin_project]
#[must_use = "futures do nothing unless polled"]
pub struct TryTimedFuture<Fut>(#[pin] TimedFuture<Fut>)
where
    Fut: TryFuture;

impl<Fut, T, E> Future for TryTimedFuture<Fut>
where
    Fut: Future<Output = Result<T, E>>,
{
    type Output = Result<Timed<T>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.0
            .poll(cx)
            .map(<TimedFuture<Fut> as Future>::Output::transpose)
    }
}

pub trait TryFutureExt: TryFuture + Sized {
    fn try_timed(self) -> TryTimedFuture<Self> {
        TryTimedFuture(self.timed())
    }
}

impl<F> TryFutureExt for F where F: TryFuture {}

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct TimedStream<St>(#[pin] St)
where
    St: Stream;

impl<St> Stream for TimedStream<St>
where
    St: Stream,
{
    type Item = Timed<<St as Stream>::Item>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.0.poll_next(cx).map(|o| o.map(Timed::new))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub trait StreamExt: Stream + Sized {
    fn timed(self) -> TimedStream<Self> {
        TimedStream(self)
    }
}

impl<St> StreamExt for St where St: Stream {}

#[pin_project]
pub struct TryTimedStream<St>(#[pin] TimedStream<St>)
where
    St: TryStream;

impl<St, T, E> Stream for TryTimedStream<St>
where
    St: Stream<Item = Result<T, E>>,
{
    type Item = Result<Timed<T>, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.0
            .poll_next(cx)
            .map(|o| o.map(<TimedStream<St> as Stream>::Item::transpose))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub trait TryStreamExt: TryStream + Sized {
    fn try_timed(self) -> TryTimedStream<Self> {
        TryTimedStream(self.timed())
    }
}

impl<St> TryStreamExt for St where St: TryStream {}
