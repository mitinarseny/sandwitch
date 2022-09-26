use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::{BufferUnordered, Inspect};
use futures::{Future, Stream};
use metrics::Counter;

pub struct BufferCounterUnordered<St>
where
    St: Stream,
{
    stream: BufferUnordered<Inspect<St, fn(&St::Item)>>,
    counter: Counter,
}

impl<St> Stream for BufferCounterUnordered<St>
where
    St: Stream,
    St::Item: Future,
{
    type Item = <BufferUnordered<St> as Stream>::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // self.stream
        todo!()
    }
}
