macro_rules! load_contract {
    ($name:ident, $file:literal) => {
        lazy_static::lazy_static! {
            static ref $name: web3::ethabi::Contract = web3::ethabi::Contract::load(include_bytes!($file).as_slice()).unwrap();
        }
    };
    (pub $name:ident, $file:literal) => {
        lazy_static::lazy_static! {
            pub static ref $name: web3::ethabi::Contract = web3::ethabi::Contract::load(include_bytes!($file).as_slice()).unwrap();
        }
    };
}

macro_rules! contract_function {
    ($name:ident, $contract:ident, $func:literal) => {
        lazy_static::lazy_static! {
            static ref $name: web3::ethabi::Function = $contract.function($func).unwrap().clone();
        }
    };
    (pub $name:ident, $contract:ident, $func:literal) => {
        lazy_static::lazy_static! {
            pub static ref $name: web3::ethabi::Function = $contract.function($func).unwrap().clone();
        }
    };
}

macro_rules! address {
    ($name:ident, $addr:literal) => {
        const $name: web3::types::Address = web3::ethabi::ethereum_types::H160(hex_literal::hex!($addr));
    };
    (pub $name:ident, $addr:literal) => {
        pub const $name: web3::types::Address = web3::ethabi::ethereum_types::H160(hex_literal::hex!($addr));
    };

}

pub mod pancake_swap;
