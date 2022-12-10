use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::future::FusedFuture;
use futures::{Future, TryFuture};
use pin_project::pin_project;

#[derive(Clone, Copy, Debug)]
pub struct With<T, U> {
    inner: T,
    with: U,
}

impl<T, U> Deref for With<T, U> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, U> DerefMut for With<T, U> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T, U> With<T, U> {
    #[inline]
    pub fn new_with(value: T, with: U) -> Self {
        Self { inner: value, with }
    }

    #[inline]
    pub fn new_with_default(value: T) -> Self
    where
        U: Default,
    {
        Self::new_with(value, <U as Default>::default())
    }

    #[inline]
    pub fn as_ref(&self) -> With<&T, &U> {
        With::new_with(&self.inner, &self.with)
    }

    #[inline]
    pub fn as_mut(&mut self) -> With<&mut T, &mut U> {
        With::new_with(&mut self.inner, &mut self.with)
    }

    #[inline]
    pub fn as_deref(&self) -> With<&<T as Deref>::Target, &U>
    where
        T: Deref,
    {
        With::new_with(self.inner.deref(), &self.with)
    }

    #[inline]
    pub fn as_deref_mut(&mut self) -> With<&mut <T as Deref>::Target, &mut U>
    where
        T: DerefMut,
    {
        With::new_with(self.inner.deref_mut(), &mut self.with)
    }

    #[inline]
    pub fn with(&self) -> &U {
        &self.with
    }

    pub fn with_mut(&mut self) -> &mut U {
        &mut self.with
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.inner
    }

    #[inline]
    pub fn into_with(self) -> U {
        self.with
    }

    #[inline]
    pub fn into_tuple(self) -> (T, U) {
        (self.inner, self.with)
    }

    #[inline]
    pub fn set<A>(self, value: A) -> With<A, U> {
        With::new_with(value, self.with)
    }

    #[inline]
    pub fn map<A, F>(self, f: F) -> With<A, U>
    where
        F: FnOnce(T) -> A,
    {
        With::new_with(f(self.inner), self.with)
    }

    #[inline]
    pub fn set_with<B>(self, with: B) -> With<T, B> {
        With::new_with(self.inner, with)
    }

    #[inline]
    pub fn map_with<B, F>(self, f: F) -> With<T, B>
    where
        F: FnOnce(U) -> B,
    {
        With::new_with(self.inner, f(self.with))
    }
}

impl<T, U> With<&T, U> {
    pub fn copied(self) -> With<T, U>
    where
        T: Copy,
    {
        With::new_with(*self.inner, self.with)
    }

    pub fn cloned(self) -> With<T, U>
    where
        T: Clone,
    {
        self.map(Clone::clone)
    }
}

impl<T, U> With<&mut T, U> {
    pub fn copied(self) -> With<T, U>
    where
        T: Copy,
    {
        With::new_with(*self.inner, self.with)
    }

    pub fn cloned(self) -> With<T, U>
    where
        T: Clone,
    {
        self.map(|v| v.clone())
    }
}

impl<T, U> With<Option<T>, U> {
    pub fn transpose(self) -> Option<With<T, U>> {
        self.inner.map(|v| With::new_with(v, self.with))
    }
}

impl<T, U, E> With<Result<T, E>, U> {
    pub fn transpose(self) -> Result<With<T, U>, E> {
        self.inner.map(|v| With::new_with(v, self.with))
    }
}

#[pin_project]
#[must_use = "futures do nothing unless polled"]
pub struct WithFuture<Fut, T>
where
    Fut: Future,
{
    #[pin]
    inner: Fut,
    with: Option<T>,
}

impl<Fut, T> Future for WithFuture<Fut, T>
where
    Fut: Future,
{
    type Output = With<<Fut as Future>::Output, T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        this.inner
            .poll(cx)
            .map(move |v| With::new_with(v, this.with.take().expect("poll called after ready")))
    }
}

impl<Fut, T> FusedFuture for WithFuture<Fut, T>
where
    Fut: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.with.is_none()
    }
}

pub trait FutureExt: Future + Sized {
    fn with<T>(self, with: T) -> WithFuture<Self, T> {
        WithFuture {
            inner: self,
            with: Some(with),
        }
    }
}

impl<F> FutureExt for F where F: Future {}

#[pin_project]
#[must_use = "futures do nothing unless polled"]
pub struct TryWithFuture<Fut, T>(#[pin] WithFuture<Fut, T>)
where
    Fut: TryFuture;

impl<Fut, U, T, E> Future for TryWithFuture<Fut, U>
where
    Fut: Future<Output = Result<T, E>>,
{
    type Output = Result<With<T, U>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.0
            .poll(cx)
            .map(<WithFuture<Fut, U> as Future>::Output::transpose)
    }
}

impl<Fut, U, T, E> FusedFuture for TryWithFuture<Fut, U>
where
    Fut: FusedFuture<Output = Result<T, E>>,
{
    fn is_terminated(&self) -> bool {
        self.0.is_terminated()
    }
}

pub trait TryFutureExt: TryFuture + Sized {
    fn try_with<T>(self, with: T) -> TryWithFuture<Self, T> {
        TryWithFuture(self.with(with))
    }
}

impl<F> TryFutureExt for F where F: TryFuture {}
