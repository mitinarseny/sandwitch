use core::future::Future;
use core::marker;
use core::pin::Pin;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::{self, Debug, Formatter};
use std::num::TryFromIntError;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ethers::providers::JsonRpcClient;

#[derive(Default, Debug)]
struct AtomicDurationMs(AtomicU64);

impl TryFrom<Duration> for AtomicDurationMs {
    type Error = TryFromIntError;

    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        Ok(Self(AtomicU64::new(value.as_millis().try_into()?)))
    }
}

impl AtomicDurationMs {
    fn as_duration(&self) -> Duration {
        Duration::from_millis(self.0.load(Ordering::SeqCst))
    }

    fn set(&self, v: Duration) {
        self.0.store(v.as_millis(), Ordering::SeqCst)
    }
}

pub struct LatencyProvider<P: JsonRpcClient> {
    inner: P,
    lattency: Arc<AtomicDurationMs>,
}

impl<P: JsonRpcClient> Deref for LatencyProvider<P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<P: JsonRpcClient> DerefMut for LatencyProvider<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<P: JsonRpcClient> From<P> for LatencyProvider<P> {
    fn from(value: P) -> Self {
        Self {
            inner: value,
            lattency: 0,
        }
    }
}

impl<P: JsonRpcClient> LatencyProvider<P> {
    pub(crate) fn new(client: P) -> Self {
        Self::from(client)
    }
}

impl<P> JsonRpcClient for LatencyProvider<P>
where
    P: JsonRpcClient,
{
    type Error = P::Error;

    fn request<'life0, 'life1, 'async_trait, T, R>(
        &'life0 self,
        method: &'life1 str,
        params: T,
    ) -> Pin<Box<dyn Future<Output = Result<R, Self::Error>> + marker::Send + 'async_trait>>
    where
        T: Debug + Serialize + Send + Sync,
        R: DeserializeOwned,
        T: 'async_trait,
        R: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        self.inner
    }
}
