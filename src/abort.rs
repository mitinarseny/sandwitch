use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use std::{
    borrow::Borrow,
    ops::{Deref, DerefMut},
};

use futures::{
    future::{select, AbortHandle, AbortRegistration, Abortable, Either, Map, Select},
    Future, FutureExt as StdFutureExt, Stream,
};

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
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn try_insert(&mut self, id: ID) -> Option<AbortRegistration> {
        match self.0.entry(id) {
            Entry::Vacant(e) => {
                let (h, reg) = AbortHandle::new_pair();
                e.insert(h);
                Some(reg)
            }
            Entry::Occupied(_) => None,
        }
    }

    pub fn abort<Q: ?Sized>(&mut self, id: &Q) -> Option<ID>
    where
        ID: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.0.remove_entry(id.borrow()).map(|(id, h)| {
            h.abort();
            id
        })
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
