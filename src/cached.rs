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
