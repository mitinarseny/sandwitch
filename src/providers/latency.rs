use core::{future::Future, pin::Pin};
use std::{fmt::Debug, time::Duration};

use ethers::{
    providers::{JsonRpcClient, PubsubClient},
    types::U256,
};
use fixed_vec_deque::FixedVecDeque;
use futures::{lock::Mutex, FutureExt};
use nalgebra as na;
use serde::{de::DeserializeOwned, Serialize};
use smartcore::{
    linalg::basic::matrix::DenseMatrix,
    linear::linear_regression::{
        LinearRegression, LinearRegressionParameters, LinearRegressionSolverName,
    },
};

use crate::timed::TryFutureExt as TimedTryFutureExt;

#[derive(Debug)]
pub struct LatencyProvider<P> {
    inner: P,
    latencies: Mutex<FixedVecDeque<[Duration; 2048]>>,
}

impl<P> LatencyProvider<P> {
    pub fn new(inner: P) -> Self {
        Self {
            inner,
            latencies: Default::default(), // TODO
        }
    }

    async fn on_elapsed(&self, elapsed: Duration) {
        // let latencies = self.latencies.lock().await;
        // *latencies.push_back() = elapsed;
    }

    pub async fn latency(&self) -> Duration {
        // TODO
        Duration::from_millis(200)
    }

    // fn latency_(durations: &[Duration]) -> Duration {
    //     let x = na::DVector::from_column_slice(durations);
    // }
}

struct OUProcess {
    alpha: f64,
    gamma: f64,
    beta: f64,
}

impl OUProcess {
    fn estimate(samples: &[f64]) -> Self {
        // DenseMatrix::from_2d_array()
        // let x = na::DVector::from_column_slice(samples);
        // let y = x[1..] - x[..-1];

        // LinearRegression::fit(
        //     &x,
        //     &y,
        //     LinearRegressionParameters::default().with_solver(LinearRegressionSolverName::QR),
        // );
        Self {
            alpha: todo!(),
            gamma: todo!(),
            beta: todo!(),
        }
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
        async move {
            let (r, elapsed) = self.inner.request(method, params).try_timed().await?;
            self.on_elapsed(elapsed).await;
            Ok(r)
        }
        .boxed()
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
