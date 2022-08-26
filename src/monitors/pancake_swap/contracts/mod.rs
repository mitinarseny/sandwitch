macro_rules! address {
    ($name:ident, $addr:literal) => {
        const $name: web3::types::Address = web3::ethabi::ethereum_types::H160(hex_literal::hex!($addr));
    };
    (pub $name:ident, $addr:literal) => {
        pub const $name: web3::types::Address = web3::ethabi::ethereum_types::H160(hex_literal::hex!($addr));
    };

}


pub mod factory_v2;
pub mod router_v2;
pub mod pair;
pub mod token;
