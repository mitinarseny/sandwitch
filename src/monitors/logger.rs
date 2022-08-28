use std::convert::Infallible;
use std::fmt::Debug;
use std::marker::PhantomData;

use super::Monitor;
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use futures::StreamExt;

pub struct Logger<T> {
    _phantom: PhantomData<T>,
}

impl<T> Logger<T> {
    pub fn new() -> Self {
        Logger {
            _phantom: PhantomData,
        }
    }
}

impl<T: Debug + 'static> Monitor for Logger<T> {
    type Item = T;
    type Error = Infallible;

    fn process(
        self: Box<Self>,
        mut stream: BoxStream<'_, Self::Item>,
    ) -> BoxFuture<'_, Result<(), Self::Error>> {
        Box::pin(async move {
            while let Some(v) = stream.next().await {
                println!("{v:?}");
            }
            Ok(())
        })
    }
}
