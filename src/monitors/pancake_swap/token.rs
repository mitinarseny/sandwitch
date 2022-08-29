use std::fmt::Display;

use anyhow::Context;
use futures::try_join;
use num::rational::Ratio;
use num::BigUint;
use web3::api::Eth;
use web3::types::Address;
use web3::Transport;

use super::contracts;

#[derive(Clone)]
pub struct Token<T: Transport> {
    contract: contracts::Token<T>,
    name: String,
    decimals: u8,
}

impl<T: Transport> Token<T> {
    pub async fn new(eth: Eth<T>, address: Address) -> anyhow::Result<Self> {
        let contract = contracts::Token::new(eth, address);
        let (name, decimals) = try_join!(contract.name(), contract.decimals())?;
        Ok(Self {
            contract,
            name,
            decimals,
        })
    }

    pub fn address(&self) -> Address {
        self.contract.address()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn as_decimals(&self, v: impl Into<u128>) -> f64 {
        v.into() as f64 / 10f64.powi(self.decimals as i32)
    }

    pub fn to_decimals(&self, v: impl Into<Ratio<BigUint>>) -> BigUint {
        (v.into() * BigUint::from(10u8).pow(self.decimals as u32)).to_integer()
    }
}

impl<T: Transport> PartialEq for Token<T> {
    fn eq(&self, other: &Self) -> bool {
        self.address().eq(&other.address())
    }
}

impl<T: Transport> PartialOrd for Token<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.address().partial_cmp(&other.address())
    }
}

impl<T: Transport> Display for Token<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name(), self.address())
    }
}

// #[derive(Clone)]
// pub struct Token {
//     address: Address,
//     name: String,
//     decimals: u8,
// }
//
// impl Token {
//     pub async fn from_address<T: Transport>(
//         eth: Eth<T>,
//         address: Address,
//     ) -> web3::contract::Result<Self> {
//         let c = Contract::new(eth, address.clone(), contracts::token::TOKEN.clone());
//         let (decimals, name) = try_join!(
//             c.query("decimals", (), None, Options::default(), None)
//                 .map_ok(|(d,)| d),
//             c.query("name", (), None, Options::default(), None)
//                 .map_ok(|(s,)| s)
//         )?;
//         Ok(Self {
//             address,
//             name,
//             decimals,
//         })
//     }
//
//     pub fn address(&self) -> Address {
//         self.address
//     }
// }
//
// impl Hash for Token {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.address.hash(state)
//     }
// }
//
// impl PartialEq for Token {
//     fn eq(&self, other: &Self) -> bool {
//         self.address.eq(&other.address)
//     }
// }
//
// impl PartialOrd for Token {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         self.address.partial_cmp(&other.address)
//     }
// }
//
// impl Eq for Token {}
