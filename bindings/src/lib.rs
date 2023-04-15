#![feature(iterator_try_collect)]

pub(crate) mod utils;

pub(crate) mod prelude {
    pub use crate::utils::*;
    use ethers::{
        abi::{AbiDecode, AbiEncode},
        contract::{ContractRevert, EthCall},
    };

    #[allow(unused_macros)]
    macro_rules! tracked_abigen {
        ($name:ident, $path:literal $(, $other:expr)*) => {
            const _: &'static str = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"), "/../", $path));
            ::ethers::contract::abigen!($name, $path $(, $other)*);
        };
    }
    pub(crate) use tracked_abigen;

    pub trait EthTypedCall: EthCall {
        type Ok: AbiEncode + AbiDecode;
        type Reverted: ContractRevert;

        fn encode_calldata(self) -> Vec<u8> {
            [Self::selector().as_slice(), self.encode().as_slice()].concat()
        }
    }
}
pub use prelude::EthTypedCall;

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

#[cfg(feature = "pancake_toaster")]
pub mod pancake_toaster {
    use crate::prelude::*;
    tracked_abigen!(
        PancakeToaster,
        "contracts/out/PancakeToaster.sol/PancakeToaster.json"
    );
}
