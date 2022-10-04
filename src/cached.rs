use std::ops::{Deref, DerefMut};

use futures::{Future, TryFuture, TryFutureExt};

pub struct Aption<T>(Option<T>);

impl<T> Deref for Aption<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Aption<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Default for Aption<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> From<Option<T>> for Aption<T> {
    fn from(o: Option<T>) -> Self {
        Self(o)
    }
}

impl<T> Aption<T> {
    pub fn into_inner(self) -> Option<T> {
        self.0
    }

    pub async fn get_or_insert_with<F, Fut>(&mut self, f: F) -> &mut T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        if let None = self.0 {
            self.0 = Some(f().await);
        }
        unsafe { self.0.as_mut().unwrap_unchecked() }
    }

    pub async fn get_or_try_insert_with<F, Fut, E>(&mut self, f: F) -> Result<&mut T, E>
    where
        F: FnOnce() -> Fut,
        Fut: TryFuture<Ok = T, Error = E>,
    {
        if let None = self.0 {
            self.0 = Some(f().into_future().await?);
        }
        Ok(unsafe { self.0.as_mut().unwrap_unchecked() })
    }
}
