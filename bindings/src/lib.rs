#![feature(iterator_try_collect)]

pub(crate) mod prelude {
    #[allow(unused_macros)]
    macro_rules! tracked_abigen {
        ($name:ident, $path:literal $(, $other:expr)*) => {
            const _: &'static str = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"), "/../", $path));
            ::ethers::contract::abigen!($name, $path $(, $other)*);
        };
    }
    pub(crate) use tracked_abigen;
}

pub(crate) mod utils;

#[cfg(feature = "multicall")]
pub mod multicall;

#[cfg(feature = "pancake_swap")]
pub mod pancake_swap {
    use crate::prelude::*;
    tracked_abigen!(
        PancakeRouter,
        "contracts/out/IPancakeRouter02.sol/IPancakeRouter02.json"
    );
}
