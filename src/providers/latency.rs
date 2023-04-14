use core::{future::Future, pin::Pin};
use std::{fmt::Debug, time::Duration};

use ethers::{
    providers::{JsonRpcClient, PubsubClient},
    types::U256,
};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug)]
pub struct LatencyProvider<P> {
    inner: P,
}

impl<P> LatencyProvider<P> {
    pub fn new(inner: P) -> Self {
        Self { inner }
    }

    pub fn latency(&self) -> Duration {
        // TODO
        Duration::from_millis(200)
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
    ) -> Pin<Box<dyn Future<Output = Result<R, Self::Error>> + Send + 'async_trait>>
    where
        T: Debug + Serialize + Send + Sync,
        R: DeserializeOwned + Send,
        T: 'async_trait,
        R: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        self.inner.request(method, params)
    }
}

impl<P> PubsubClient for LatencyProvider<P>
where
    P: PubsubClient,
{
    type NotificationStream = P::NotificationStream;

    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, Self::Error> {
        self.inner.subscribe(id)
    }

    fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<(), Self::Error> {
        self.inner.unsubscribe(id)
    }
}
