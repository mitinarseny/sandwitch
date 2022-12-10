use std::{
    borrow::Borrow,
    collections::{
        hash_map::{Entry, VacantEntry},
        HashMap,
    },
    hash::Hash,
    ops::{Deref, DerefMut},
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
use pin_project::pin_project;
use tokio::task::{JoinError, JoinHandle};
use tokio_util::sync::CancellationToken;

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

#[derive(Default)]
pub struct AbortSet<ID>(HashMap<ID, AbortHandle>)
where
    ID: Eq + Hash;

impl<ID> Deref for AbortSet<ID>
where
    ID: Eq + Hash,
{
    type Target = HashMap<ID, AbortHandle>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<ID> DerefMut for AbortSet<ID>
where
    ID: Eq + Hash,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<ID> AbortSet<ID>
where
    ID: Eq + Hash,
{
    pub fn try_insert(&mut self, id: ID) -> Result<AbortRegistration, ID>
    where
        ID: Clone,
    {
        match self.0.entry(id) {
            Entry::Vacant(e) => {
                let (h, reg) = AbortHandle::new_pair();
                e.insert(h);
                Ok(reg)
            }
            Entry::Occupied(e) => Err(e.key().clone()),
        }
    }

    pub fn abort<Q: ?Sized>(&mut self, id: &Q) -> Option<ID>
    where
        ID: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.0.remove_entry(id).map(|(id, h)| {
            h.abort();
            id
        })
    }

    pub fn abort_iter<'a, Q: ?Sized + 'a>(
        &'a mut self,
        ids: impl IntoIterator<Item = &'a Q> + 'a,
    ) -> impl Iterator<Item = ID> + 'a
    where
        ID: Borrow<Q>,
        Q: Eq + Hash,
    {
        ids.into_iter().filter_map(|id| self.abort(id))
    }

    pub fn abort_all(&mut self) -> impl Iterator<Item = ID> + '_ {
        self.0.drain().map(|(id, h)| {
            h.abort();
            id
        })
    }
}

impl<ID> Drop for AbortSet<ID>
where
    ID: Eq + Hash,
{
    fn drop(&mut self) {
        self.abort_all().for_each(drop);
    }
}

pub struct CancelSet<ID> {
    root: CancellationToken,
    m: HashMap<ID, CancellationToken>,
}

impl<ID> CancelSet<ID>
where
    ID: Eq + Hash,
{
    pub fn new(root: CancellationToken) -> Self {
        Self {
            root,
            m: HashMap::new(),
        }
    }

    pub fn try_insert(&mut self, id: ID) -> Result<CancellationToken, ID>
    where
        ID: Clone,
    {
        match self.m.entry(id) {
            Entry::Vacant(e) => Ok(e.insert(self.root.child_token()).clone()),
            Entry::Occupied(e) => Err(e.key().clone()),
        }
    }

    pub fn cancel<Q: ?Sized>(&mut self, id: &Q) -> Option<ID>
    where
        ID: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.m.remove_entry(id).map(|(id, c)| {
            c.cancel();
            id
        })
    }
}

impl<ID> Drop for CancelSet<ID> {
    fn drop(&mut self) {
        self.root.cancel();
        self.m.clear();
    }
}

#[pin_project]
#[must_use = "futures do nothing unless polled"]
pub struct JoinHandleSet<ID, T> {
    #[pin]
    futs: FuturesUnordered<JoinHandle<(Result<T, Aborted>, ID)>>,
    aborts: HashMap<ID, AbortHandle>,
}

impl<ID, T> Default for JoinHandleSet<ID, T> {
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
                    this.aborts.remove(&id).expect("TODO"); // TODO
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

// impl<ID, T> Drop for JoinHandleSet<ID, T> {
//     fn drop(&mut self) {
//         self.0
//             .drain()
//             .map(|(_id, h)| {
//                 h.abort();
//             })
//             .for_each(drop)
//     }
// }

// #[derive(Default)]
// pub struct AbortMap<ID, T>(HashMap<ID, (T, AbortHandle)>)
// where
//     ID: Eq + Hash;
//
// impl<ID, T> AbortMap<ID, T>
// where
//     ID: Eq + Hash,
// {
//     pub fn try_insert(&mut self, id: ID, value: T) -> Result<AbortRegistration, (ID, T)> {
//         match self.0.entry(id) {
//             Entry::Vacant(e) => {
//                 let (h, reg) = AbortHandle::new_pair();
//                 e.insert((value, h));
//                 Ok(reg)
//             }
//             Entry::Occupied(e) => Err((id, value)),
//         }
//     }
//
//     pub fn abort<Q: ?Sized>(&mut self, id: &Q) -> Option<(ID, T)>
//     where
//         ID: Borrow<Q>,
//         Q: Eq + Hash,
//     {
//         self.0.remove_entry(id).map(|(id, (v, h))| {
//             h.abort();
//             (id, v)
//         })
//     }
//
//     pub fn abort_all(&mut self) -> impl Iterator<Item = (ID, T)> + '_ {
//         self.0.drain().map(|(id, (v, h))| {
//             h.abort();
//             (id, v)
//         })
//     }
// }

// pub struct AbortSet<ID>(Mutex<LockedAbortSet<ID>>)
// where
//     ID: Eq + Hash;
//
// impl<ID> AbortSet<ID>
// where
//     ID: Eq + Hash,
// {
//     pub fn new() -> Self {
//         Self(Mutex::new(LockedAbortSet::new()))
//     }
//
//     pub async fn get(&self) -> MutexGuard<'_, LockedAbortSet<ID>> {
//         self.0.lock().await
//     }
//
//     pub fn get_mut(&mut self) -> &mut LockedAbortSet<ID> {
//         self.0.get_mut()
//     }
//
//     pub async fn try_insert(&self, id: ID) -> Option<AbortRegistration> {
//         self.0.lock().await.try_insert(id)
//     }
//
//     pub async fn abort<Q: ?Sized>(&self, id: &Q) -> Option<ID>
//     where
//         ID: Borrow<Q>,
//         Q: Eq + Hash,
//     {
//         self.0.lock().await.abort(id)
//     }
//
//     pub async fn abort_iter<'a, Q: ?Sized>(
//         &'a self,
//         ids: impl IntoIterator<Item = &'a Q> + 'a,
//     ) -> impl Iterator<Item = ID> + 'a
//     where
//         ID: Borrow<Q>,
//         Q: Eq + Hash + 'a,
//     {
//         let mut set = self.0.lock().await;
//         ids.into_iter()
//             .map(Borrow::borrow)
//             .filter_map(move |id| set.abort(id))
//     }
//
//     pub async fn abort_all<B: FromIterator<ID>>(&self) -> B {
//         self.0.lock().await.abort_all().collect()
//     }
// }
