use std::{
    borrow::Borrow,
    collections::{
        hash_map::{Entry, VacantEntry},
        HashMap,
    },
    hash::Hash,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{
    future::{
        select, AbortHandle, AbortRegistration, Abortable, Aborted, Either, Future,
        FutureExt as StdFutureExt, Map, Select,
    },
    stream::{FusedStream, FuturesUnordered, Stream},
};
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

#[pin_project(PinnedDrop)]
#[must_use = "futures do nothing unless polled"]
pub struct JoinHandleSet<ID, T>
where
    ID: Eq + Hash,
{
    #[pin]
    futs: FuturesUnordered<JoinHandle<(Result<T, Aborted>, ID)>>,
    aborts: HashMap<ID, AbortHandle>,
}

impl<ID, T> Default for JoinHandleSet<ID, T>
where
    ID: Eq + Hash,
{
    fn default() -> Self {
        Self {
            futs: FuturesUnordered::new(),
            aborts: HashMap::new(),
        }
    }
}

pub struct JoinEntry<'a, ID, T> {
    futs: &'a mut FuturesUnordered<JoinHandle<(Result<T, Aborted>, ID)>>,
    abort_entry: VacantEntry<'a, ID, AbortHandle>,
}

impl<'a, ID, T> JoinEntry<'a, ID, T> {
    pub fn spawn<Fut>(self, f: Fut)
    where
        ID: Clone + Send + 'static,
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let (h, reg) = AbortHandle::new_pair();
        let id = self.abort_entry.insert_entry(h).key().clone();
        self.futs
            .push(tokio::spawn(f.with_abort_reg(reg).map(move |v| (v, id))));
    }
}

impl<ID, T> JoinHandleSet<ID, T>
where
    ID: Eq + Hash,
{
    pub fn try_insert(&mut self, id: ID) -> Result<JoinEntry<'_, ID, T>, ID>
    where
        ID: Clone,
    {
        match self.aborts.entry(id) {
            Entry::Vacant(e) => Ok(JoinEntry {
                futs: &mut self.futs,
                abort_entry: e,
            }),
            Entry::Occupied(e) => Err(e.key().clone()),
        }
    }

    pub fn abort<Q>(&mut self, id: &Q) -> Option<ID>
    where
        ID: Borrow<Q>,
        Q: Eq + Hash,
    {
        let (id, h) = self.aborts.remove_entry(id)?;
        h.abort();
        Some(id)
    }

    pub fn abort_all(&mut self) {
        self.aborts.drain().for_each(|(_id, h)| {
            h.abort();
        });
    }
}

impl<ID, T> Stream for JoinHandleSet<ID, T>
where
    ID: Eq + Hash,
{
    type Item = Result<T, JoinError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.futs.poll_next(cx).ready()? {
            Some(r) => match r {
                Ok((r, id)) => {
                    this.aborts.remove(&id);
                    match r {
                        Ok(v) => Poll::Ready(Some(Ok(v))),
                        Err(Aborted) => Poll::Pending,
                    }
                }
                Err(e) => Poll::Ready(Some(Err(e))),
            },
            None => Poll::Ready(None),
        }
    }
}

impl<ID, T> FusedStream for JoinHandleSet<ID, T>
where
    ID: Eq + Hash,
{
    fn is_terminated(&self) -> bool {
        self.futs.is_terminated()
    }
}

#[pinned_drop]
impl<ID, T> PinnedDrop for JoinHandleSet<ID, T>
where
    ID: Eq + Hash,
{
    fn drop(self: Pin<&mut Self>) {
        self.get_mut().abort_all()
    }
}
