use std::{collections::HashMap, hash::Hash, sync::Arc};

use ethers::types::H256;
use futures::{lock::Mutex, Future, TryFuture, TryFutureExt};

#[derive(Default)]
pub struct Cached<T>(Mutex<Option<T>>);

impl<T, O: Into<Option<T>>> From<O> for Cached<T> {
    fn from(o: O) -> Self {
        Self(Mutex::new(o.into()))
    }
}

impl<T> AsMut<Option<T>> for Cached<T> {
    fn as_mut(&mut self) -> &mut Option<T> {
        self.0.get_mut()
    }
}

impl<T> Cached<T> {
    #[allow(dead_code)]
    pub async fn map<R, F>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.0.lock().await.as_mut().map(f)
    }

    #[allow(dead_code)]
    pub async fn then<F, Fut>(&self, f: F) -> Option<Fut::Output>
    where
        F: FnOnce(&mut T) -> Fut,
        Fut: Future,
    {
        let mut v = self.0.lock().await;
        let v = v.as_mut()?;
        Some(f(v).await)
    }

    #[allow(dead_code)]
    pub async fn get_or_insert_with_map<F, Fut, M, R>(&self, f: F, m: M) -> R
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
        M: FnOnce(&mut T) -> R,
    {
        let mut g = self.0.lock().await;
        if let None = *g {
            *g = Some(f().await);
        }
        m(unsafe { g.as_mut().unwrap_unchecked() })
    }

    #[allow(dead_code)]
    pub async fn get_or_insert_with<F, Fut>(&self, f: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
        T: Clone,
    {
        self.get_or_insert_with_map(f, |v| v.clone()).await
    }

    #[allow(dead_code)]
    pub async fn get_or_try_insert_with_map<F, Fut, M, R>(
        &self,
        f: F,
        m: M,
    ) -> Result<R, Fut::Error>
    where
        F: FnOnce() -> Fut,
        Fut: TryFuture<Ok = T>,
        M: FnOnce(&mut T) -> R,
    {
        let mut g = self.0.lock().await;
        if let None = *g {
            *g = Some(f().into_future().await?);
        }
        Ok(m(unsafe { g.as_mut().unwrap_unchecked() }))
    }

    #[allow(dead_code)]
    pub async fn get_or_try_insert_with<F, Fut>(&self, f: F) -> Result<T, Fut::Error>
    where
        F: FnOnce() -> Fut,
        Fut: TryFuture<Ok = T>,
        T: Clone,
    {
        self.get_or_try_insert_with_map(f, |v| v.clone()).await
    }

    #[allow(dead_code)]
    pub async fn flush(&self) {
        *self.0.lock().await = None
    }
}

#[derive(Default)]
pub struct CachedAt<ID, T>(Mutex<HashMap<ID, Arc<Cached<T>>>>);

impl<ID, T> CachedAt<ID, T>
where
    ID: Eq + Hash + Clone,
    T: Clone,
{
    async fn get_at(&self, at: ID) -> Arc<Cached<T>> {
        let mut m = self.0.lock().await;
        // TODO: drain old entries
        m.entry(at).or_insert_with(|| Arc::new(None.into())).clone()
    }

    #[allow(dead_code)]
    pub async fn get_at_or_insert_with<F, Fut>(&self, at: ID, f: F) -> T
    where
        F: FnOnce(&ID) -> Fut,
        Fut: Future<Output = T>,
    {
        self.get_at(at.clone())
            .await
            .get_or_insert_with(|| f(&at))
            .await
    }

    #[allow(dead_code)]
    pub async fn get_at_or_try_insert_with<F, Fut>(&self, at: ID, f: F) -> Result<T, Fut::Error>
    where
        F: FnOnce(&ID) -> Fut,
        Fut: TryFuture<Ok = T>,
    {
        self.get_at(at.clone())
            .await
            .get_or_try_insert_with(|| f(&at))
            .await
    }

    #[allow(dead_code)]
    pub async fn retain<F>(&self, mut pred: F)
    where
        F: FnMut(&ID) -> bool,
    {
        let mut m = self.0.lock().await;
        m.retain(|id, _v| pred(id));
        m.shrink_to_fit();
    }

    #[allow(dead_code)]
    pub async fn flush(&self) {
        let mut m = self.0.lock().await;
        m.clear();
        m.shrink_to_fit();
    }
}

pub type CachedAtBlock<T> = CachedAt<H256, T>;

// #[derive(Default)]
// pub struct CachedAt<T: Clone, ID, const N: usize>(RwLock<VecDeque<(T, ID)>>);
//
// impl<T: Clone, ID, const N: usize> CachedAt<T, ID, N> {
//     pub async fn get_at_or_insert_with<F, Fut>(&self, at: ID, f: F) -> T
//     where
//         F: FnMut(&ID) -> Fut,
//         Fut: Future<Output = T>,
//     {
//         let vals = self.0.read().await;
//         match vals.binary_search_by_key(at, |v| &v.1) {
//             Ok(i) => Ok(vals[i].0.clone()),
//             Err(i) => {
//                 drop(vals);
//                 let mut vals = self.0.write().await;
//                 let v = f(&at).await;
//                 vals.insert(i, v.clone());
//                 Ok(v)
//             }
//         }
//     }
//
//     pub async fn try_get_at_or_insert_with<F, Fut>(&self, at: ID, f: F) -> Result<T, Fut::Error>
//     where
//         F: FnMut(&ID) -> Fut,
//         Fut: TryFuture<Ok = T>,
//     {
//         let vals = self.0.read().await;
//         match vals.binary_search_by_key(at, |v| &v.1) {
//             Ok(i) => Ok(vals[i].0.clone()),
//             Err(i) => {
//                 drop(vals);
//                 let mut vals = self.0.write().await;
//                 // TODO: do not lock untiil the value is not resolved
//                 // but it is important to avoid identical requests
//                 // so that only one value is resolved at time
//                 let v = f(&at).into_future().await?;
//                 vals.insert(i, v.clone());
//                 Ok(v)
//             }
//         }
//     }
// }

// #[derive(Default)]
// pub struct CachedAt<T, ID>(Option<With<T, ID>>);
//
// impl<T, ID, W> From<W> for CachedAt<T, ID>
// where
//     W: Into<Option<With<T, ID>>>,
// {
//     fn from(o: W) -> Self {
//         Self(o.into())
//     }
// }
//
// impl<T, ID: PartialEq> CachedAt<T, ID> {
//     #[inline]
//     pub fn as_ref(&self) -> CachedAt<&T, &ID> {
//         CachedAt(self.0.as_ref().map(With::as_ref))
//     }
//
//     #[inline]
//     pub fn into_inner(self) -> Option<With<T, ID>> {
//         self.0
//     }
//
//     #[inline]
//     pub fn at(&self) -> Option<&ID> {
//         self.0.as_ref().map(AsRef::<ID>::as_ref)
//     }
//
//     pub async fn get_at_or_insert_with<'a, F, Fut>(&'a mut self, id: ID, f: F) -> &'a mut T
//     where
//         F: FnOnce(&ID) -> Fut,
//         Fut: Future<Output = T>,
//     {
//         if !self.at().map_or(false, |cached_at| id == *cached_at) {
//             // TODO: what if id < cached_at?
//             self.0 = Some(With::new_with(f(&id).await, id));
//         }
//         unsafe { self.0.as_mut().map(DerefMut::deref_mut).unwrap_unchecked() }
//     }
//
//     pub async fn get_at_or_try_insert_with<F, Fut>(
//         &mut self,
//         id: ID,
//         f: F,
//     ) -> Result<&mut T, Fut::Error>
//     where
//         F: FnOnce(&ID) -> Fut,
//         Fut: TryFuture<Ok = T>,
//     {
//         if !self.at().map_or(false, |cached_at| id == *cached_at) {
//             self.0 = Some(With::new_with(f(&id).into_future().await?, id));
//         }
//         Ok(unsafe { self.0.as_mut().map(DerefMut::deref_mut).unwrap_unchecked() })
//     }
//
//     pub fn flush(&mut self) {
//         self.0 = None
//     }
// }
