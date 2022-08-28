use std::convert::Infallible;
use std::fmt::Debug;
use std::future;
use std::marker::PhantomData;

use super::Monitor;
use futures::future::BoxFuture;
use futures::FutureExt;

// pub struct Logger<T> {
//     _phantom: PhantomData<T>,
// }
//
// impl<T> Logger<T> {
//     pub fn new() -> Self {
//         Logger {
//             _phantom: PhantomData,
//         }
//     }
// }
//
// impl<T: Debug + 'static> Monitor<T> for Logger<T> {
//     type Error = Infallible;
//
//     fn process(&mut self, stream) -> BoxFuture<'_, Result<(), Self::Error>> {
//         println!("{item:?}");
//         future::ready(Ok(())).boxed()
//     }
// }
