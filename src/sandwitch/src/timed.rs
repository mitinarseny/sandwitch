use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Future, Stream, TryFuture, TryStream};
use pin_project::pin_project;
use tokio::time::{Duration, Instant};

pub trait FutureExt: Future + Sized {
    fn timed(self) -> TimedFuture<Self> {
        TimedFuture {
            inner: self,
            start: None,
        }
    }
}

impl<F> FutureExt for F where F: Future {}

pub trait TryFutureExt: TryFuture + Sized {
    fn try_timed(self) -> TryTimedFuture<Self> {
        TryTimedFuture(self.timed())
    }
}

impl<F> TryFutureExt for F where F: TryFuture {}

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
    type Output = (Fut::Output, Duration);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let start = this.start.get_or_insert_with(Instant::now);

        this.inner.poll(cx).map(move |v| (v, start.elapsed()))
    }
}

#[pin_project]
#[must_use = "futures do nothing unless polled"]
pub struct TryTimedFuture<Fut>(#[pin] TimedFuture<Fut>)
where
    Fut: TryFuture;

impl<Fut, T, E> Future for TryTimedFuture<Fut>
where
    Fut: Future<Output = Result<T, E>>,
{
    type Output = Result<(T, Duration), E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.0.poll(cx).map(|(r, t)| r.map(move |v| (v, t)))
    }
}

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct TimedStream<St>(#[pin] St)
where
    St: Stream;

impl<St> Stream for TimedStream<St>
where
    St: Stream,
{
    type Item = (St::Item, Instant);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.0.poll_next(cx).map(|o| o.map(|v| (v, Instant::now())))
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
    type Item = Result<(T, Instant), E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.0
            .poll_next(cx)
            .map(|o| o.map(|(r, t)| r.map(move |v| (v, t))))
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
