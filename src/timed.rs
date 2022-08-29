use std::ops::Deref;
use std::pin::Pin;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures::Future;

#[derive(Clone)]
pub struct Timed<T> {
    inner: T,
    at: SystemTime,
}

#[allow(dead_code)]
impl<T> Timed<T> {
    pub fn new(inner: T) -> Self {
        Self::with(inner, SystemTime::now())
    }

    pub fn with(inner: T, at: SystemTime) -> Self {
        Self { inner, at }
    }

    pub fn at(&self) -> SystemTime {
        self.at
    }

    pub fn unix(&self) -> Duration {
        self.at.duration_since(UNIX_EPOCH).unwrap()
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn map<U, F>(self, f: F) -> Timed<U>
    where
        F: FnOnce(T) -> U,
    {
        Timed {
            inner: f(self.inner),
            at: self.at,
        }
    }
}

impl<T> Future for Timed<T>
where
    T: Future + Unpin,
{
    type Output = <T as Future>::Output;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx)
    }
}

impl<T> Deref for Timed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
