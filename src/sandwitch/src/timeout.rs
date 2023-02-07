use core::{
    fmt::{self, Debug},
    future::Future,
    marker,
    ops::{Deref, DerefMut},
    pin::Pin,
};

use ethers::{
    providers::{JsonRpcClient, ProviderError, PubsubClient},
    types::U256,
};
use futures::{FutureExt, TryFutureExt};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use tokio::time::{timeout, Duration};

#[derive(Error, Debug)]
pub enum TimeoutProviderError<P: JsonRpcClient> {
    /// Timeout exceeded
    #[error("timeout exceeded: {0:?}")]
    Timeout(Duration),

    #[error(transparent)]
    Inner(P::Error),
}

impl<P> From<TimeoutProviderError<P>> for ProviderError
where
    P: JsonRpcClient + 'static,
    P::Error: Send + Sync + 'static,
{
    fn from(value: TimeoutProviderError<P>) -> Self {
        if let TimeoutProviderError::Inner(e) = value {
            return e.into();
        }
        (Box::new(value) as Box<dyn std::error::Error + Send + Sync>).into()
    }
}

pub struct TimeoutProvider<P: JsonRpcClient> {
    inner: P,
    timeout: Duration,
}

impl<P: JsonRpcClient> Deref for TimeoutProvider<P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<P: JsonRpcClient> DerefMut for TimeoutProvider<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<P: JsonRpcClient> Debug for TimeoutProvider<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TimeoutClient")
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl<P: JsonRpcClient> TimeoutProvider<P> {
    pub(crate) fn new(client: P, timeout: Duration) -> Self {
        Self {
            inner: client,
            timeout,
        }
    }
}

impl<P> JsonRpcClient for TimeoutProvider<P>
where
    P: JsonRpcClient + 'static,
    P::Error: Send + Sync + 'static,
{
    type Error = TimeoutProviderError<P>;

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
        timeout(
            self.timeout,
            self.inner
                .request(method, params)
                .map_err(TimeoutProviderError::Inner),
        )
        .map_err(|_| TimeoutProviderError::Timeout(self.timeout))
        .map(Result::flatten)
        .boxed()
    }
}

impl<P> PubsubClient for TimeoutProvider<P>
where
    P: PubsubClient + 'static,
    P::Error: Send + Sync + 'static,
{
    type NotificationStream = P::NotificationStream;

    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, Self::Error> {
        self.inner
            .subscribe(id)
            .map_err(TimeoutProviderError::Inner)
    }

    fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<(), Self::Error> {
        self.inner
            .unsubscribe(id)
            .map_err(TimeoutProviderError::Inner)
    }
}
