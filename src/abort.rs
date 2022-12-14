use std::{
    borrow::Borrow,
    collections::{
        hash_map::{Entry, VacantEntry},
        HashMap,
    },
    convert,
    hash::Hash,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{
    future::{
        select, AbortHandle, AbortRegistration, Abortable, Aborted, Either, Future,
        FutureExt as StdFutureExt, Map, Select,
    },
    stream::{FusedStream, FuturesOrdered, FuturesUnordered, Stream},
};
use metrics::Gauge;
use pin_project::{pin_project, pinned_drop};
use tokio::task::{JoinError, JoinHandle};

pub type WithAbort<Fut, A> = Map<
    Select<A, Fut>,
    fn(
        <Select<A, Fut> as Future>::Output,
    ) -> Result<<Fut as Future>::Output, <A as Future>::Output>,
>;

pub trait FutureExt: Future + Sized {
    fn with_abort_reg(self, reg: AbortRegistration) -> Abortable<Self> {
        Abortable::new(self, reg)
    }

    fn with_abort<Fut>(self, f: Fut) -> WithAbort<Self, Fut>
    where
        Fut: Future + Unpin,
        Self: Unpin,
    {
        select(f, self).map(|either| match either {
            Either::Left((err, _)) => Err(err),
            Either::Right((r, _)) => Ok(r),
        })
    }
}

impl<Fut> FutureExt for Fut where Fut: Future {}

pub trait StreamExt: Stream + Sized {
    fn with_abort_reg(self, reg: AbortRegistration) -> Abortable<Self> {
        Abortable::new(self, reg)
    }
}

impl<St> StreamExt for St where St: Stream {}

pub(crate) trait FuturesQueue<Fut>: FusedStream<Item = Fut::Output>
where
    Fut: Future,
{
    fn push(&mut self, f: Fut);
}

impl<Fut> FuturesQueue<Fut> for FuturesUnordered<Fut>
where
    Fut: Future,
{
    fn push(&mut self, f: Fut) {
        FuturesUnordered::push(&self, f)
    }
}

impl<Fut> FuturesQueue<Fut> for FuturesOrdered<Fut>
where
    Fut: Future,
{
    fn push(&mut self, f: Fut) {
        FuturesOrdered::push_back(self, f)
    }
}

#[pin_project]
#[must_use = "futures do nothing unless polled"]
pub(crate) struct WithID<Fut, ID> {
    #[pin]
    inner: Fut,
    id: Option<ID>,
}

impl<Fut, ID> WithID<Fut, ID>
where
    Fut: Future,
{
    fn new(f: Fut, id: ID) -> Self {
        Self {
            inner: f,
            id: Some(id),
        }
    }
}

impl<Fut, ID> Future for WithID<Fut, ID>
where
    Fut: Future,
{
    type Output = (Fut::Output, ID);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let v = this.inner.poll(cx).ready()?;
        Poll::Ready((v, this.id.take().expect("cannot poll AbortableID twice")))
    }
}

pub(crate) struct AbortEntry<'a, ID, FQ, Fut>
where
    ID: Eq + Hash,
    FQ: FuturesQueue<WithID<Fut, ID>>,
    Fut: Future,
{
    futs: &'a mut FQ,
    entry: VacantEntry<'a, ID, AbortHandle>,
    _phantom_data: PhantomData<Fut>,
}

impl<'a, ID, FQ, Fut> AbortEntry<'a, ID, FQ, Fut>
where
    ID: Eq + Hash,
    FQ: FuturesQueue<WithID<Fut, ID>>,
    Fut: Future,
{
    fn insert_with<F, IFut>(self, inner: IFut, f: F)
    where
        ID: Clone,
        IFut: Future,
        F: Fn(Abortable<IFut>) -> Fut,
    {
        let (h, reg) = AbortHandle::new_pair();
        let id = self.entry.insert_entry(h).key().clone();
        self.futs
            .push(WithID::new(f(inner.with_abort_reg(reg)), id))
    }
}

impl<'a, ID, FQ, Fut> AbortEntry<'a, ID, FQ, Abortable<Fut>>
where
    ID: Eq + Hash,
    FQ: FuturesQueue<WithID<Abortable<Fut>, ID>>,
    Fut: Future,
{
    pub(crate) fn insert(self, f: Fut)
    where
        ID: Clone,
    {
        self.insert_with(f, convert::identity)
    }
}

impl<'a, ID, FQ, T> AbortEntry<'a, ID, FQ, AbortTaskGuard<Result<T, Aborted>>>
where
    ID: Eq + Hash,
    FQ: FuturesQueue<WithID<AbortTaskGuard<Result<T, Aborted>>, ID>>,
{
    pub(crate) fn spawn<Fut>(self, f: Fut)
    where
        ID: Clone,
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        self.insert_with(f, AbortTaskGuard::spawn)
    }
}

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub(crate) struct AbortSet<ID, FQ, Fut>
where
    ID: Eq + Hash,
    FQ: FuturesQueue<WithID<Fut, ID>>,
    Fut: Future,
{
    #[pin]
    futs: FQ,
    aborts: HashMap<ID, AbortHandle>,
    _phantom_data: PhantomData<Fut>,
}

impl<ID, FQ, Fut> Default for AbortSet<ID, FQ, Fut>
where
    ID: Eq + Hash,
    FQ: FuturesQueue<WithID<Fut, ID>> + Default,
    Fut: Future,
{
    fn default() -> Self {
        Self::new(FQ::default())
    }
}

impl<ID, FQ, Fut> AbortSet<ID, FQ, Fut>
where
    ID: Eq + Hash,
    FQ: FuturesQueue<WithID<Fut, ID>>,
    Fut: Future,
{
    pub(crate) fn new(futs: FQ) -> Self {
        Self {
            futs,
            aborts: HashMap::new(),
            _phantom_data: PhantomData,
        }
    }

    pub(crate) fn try_insert(&mut self, id: ID) -> Result<AbortEntry<'_, ID, FQ, Fut>, ID>
    where
        ID: Clone,
    {
        match self.aborts.entry(id) {
            Entry::Vacant(e) => Ok(AbortEntry {
                futs: &mut self.futs,
                entry: e,
                _phantom_data: PhantomData,
            }),
            Entry::Occupied(e) => Err(e.key().clone()),
        }
    }

    pub(crate) fn contains<Q>(&self, id: &Q) -> bool
    where
        ID: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.aborts.contains_key(id)
    }

    pub(crate) fn abort<Q>(&mut self, id: &Q) -> Option<ID>
    where
        ID: Borrow<Q>,
        Q: Eq + Hash,
    {
        let (id, h) = self.aborts.remove_entry(id)?;
        h.abort();
        Some(id)
    }

    pub(crate) fn abort_all(&mut self) {
        self.aborts.drain().for_each(|(_id, h)| {
            h.abort();
        });
    }

    fn poll_next_with<F, T>(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut f: F,
    ) -> Poll<Option<(T, ID)>>
    where
        F: FnMut(Fut::Output) -> Option<T>,
    {
        let mut this = self.project();
        Poll::Ready(loop {
            let Some((r, id)) = this.futs.as_mut().poll_next(cx).ready()? else {
                break None;
            };
            this.aborts.remove(&id);
            break Some((
                match f(r) {
                    Some(v) => v,
                    None => continue,
                },
                id,
            ));
        })
    }
}

impl<ID, FQ, Fut> FusedStream for AbortSet<ID, FQ, Fut>
where
    Self: Stream,
    ID: Eq + Hash,
    FQ: FuturesQueue<WithID<Fut, ID>>,
    Fut: Future,
{
    fn is_terminated(&self) -> bool {
        self.futs.is_terminated()
    }
}

impl<ID, FQ, Fut> Stream for AbortSet<ID, FQ, Abortable<Fut>>
where
    ID: Eq + Hash,
    FQ: FuturesQueue<WithID<Abortable<Fut>, ID>>,
    Fut: Future,
{
    type Item = (Fut::Output, ID);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_next_with(cx, Result::ok)
    }
}

impl<ID, FQ, T> Stream for AbortSet<ID, FQ, AbortTaskGuard<Result<T, Aborted>>>
where
    ID: Eq + Hash,
    FQ: FuturesQueue<WithID<AbortTaskGuard<Result<T, Aborted>>, ID>>,
{
    type Item = (Result<T, JoinError>, ID);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_next_with(cx, |r| r.map(Result::ok).transpose())
    }
}

#[pin_project(PinnedDrop)]
pub(crate) struct AbortTaskGuard<T>(#[pin] JoinHandle<T>);

impl<T> AbortTaskGuard<T> {
    fn spawn<Fut>(fut: Fut) -> Self
    where
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        Self(tokio::spawn(fut))
    }
}

impl<T> Future for AbortTaskGuard<T> {
    type Output = <JoinHandle<T> as Future>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().0.poll(cx)
    }
}

#[pinned_drop]
impl<T> PinnedDrop for AbortTaskGuard<T> {
    fn drop(self: Pin<&mut Self>) {
        self.project().0.as_mut().abort();
    }
}

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub(crate) struct MetricedFuturesQueue<FQ, Fut>
where
    FQ: FuturesQueue<Fut>,
    Fut: Future,
{
    #[pin]
    inner: FQ,
    gauge: Gauge,
    _fut: PhantomData<Fut>,
}

impl<FQ, Fut> MetricedFuturesQueue<FQ, Fut>
where
    FQ: FuturesQueue<Fut>,
    Fut: Future,
{
    pub fn new(inner: FQ, gauge: Gauge) -> Self {
        Self {
            inner,
            gauge,
            _fut: PhantomData,
        }
    }

    pub fn new_with_default(gauge: Gauge) -> Self
    where
        FQ: Default,
    {
        Self::new(FQ::default(), gauge)
    }
}

impl<FQ, Fut> Stream for MetricedFuturesQueue<FQ, Fut>
where
    FQ: FuturesQueue<Fut>,
    Fut: Future,
{
    type Item = Fut::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let r = this.inner.poll_next(cx).ready()?;
        if r.is_some() {
            this.gauge.decrement(1f64);
        }
        Poll::Ready(r)
    }
}

impl<FQ, Fut> FusedStream for MetricedFuturesQueue<FQ, Fut>
where
    FQ: FuturesQueue<Fut>,
    Fut: Future,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

impl<FQ, Fut> FuturesQueue<Fut> for MetricedFuturesQueue<FQ, Fut>
where
    FQ: FuturesQueue<Fut>,
    Fut: Future,
{
    fn push(&mut self, f: Fut) {
        self.inner.push(f);
        self.gauge.increment(1f64);
    }
}
