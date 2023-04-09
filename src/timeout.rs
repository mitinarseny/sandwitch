use core::{fmt::Debug, future::Future, marker, pin::Pin};

use ethers::{
    providers::{JsonRpcClient, ProviderError, PubsubClient, RpcError},
    types::U256,
};
use futures::{FutureExt, TryFutureExt};
use impl_tools::autoimpl;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error as ThisError;
use tokio::time::{timeout, Duration};

#[autoimpl(Deref using self.inner)]
#[autoimpl(DerefMut using self.inner)]
#[derive(Debug)]
pub struct TimeoutProvider<P: JsonRpcClient> {
    inner: P,
    timeout: Duration,
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
        R: DeserializeOwned + Send,
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

#[derive(ThisError, Debug)]
pub enum TimeoutProviderError<P: JsonRpcClient> {
    /// Timeout exceeded
    #[error("timeout exceeded: {0:?}")]
    Timeout(Duration),

    #[error(transparent)]
    Inner(P::Error),
}

impl<P: JsonRpcClient> TimeoutProviderError<P> {
    fn as_inner(&self) -> Option<&P::Error> {
        match self {
            TimeoutProviderError::Inner(inner) => Some(inner),
            _ => None,
        }
    }
}

impl<P: JsonRpcClient> RpcError for TimeoutProviderError<P> {
    fn as_error_response(&self) -> Option<&ethers::providers::JsonRpcError> {
        self.as_inner().map(RpcError::as_error_response).flatten()
    }

    fn as_serde_error(&self) -> Option<&serde_json::Error> {
        self.as_inner().map(RpcError::as_serde_error).flatten()
    }
}

impl<P> From<TimeoutProviderError<P>> for ProviderError
where
    P: JsonRpcClient + 'static,
    P::Error: Send + Sync + 'static,
{
    fn from(e: TimeoutProviderError<P>) -> Self {
        if let TimeoutProviderError::Inner(e) = e {
            return e.into();
        }

        ProviderError::JsonRpcClientError(Box::new(e) as Box<dyn RpcError + Send + Sync>)
    }
}
