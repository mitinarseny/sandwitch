use futures::future::BoxFuture;

pub mod pancake_swap_router_v2;

pub trait SwapContract {
    fn process(&mut self, input: &[u8]) -> BoxFuture<'_, ()>;
}
