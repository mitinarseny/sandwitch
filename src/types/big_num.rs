use byte_slice_cast::*;
use num::BigUint;

pub struct U256(pub web3::types::U256);

impl Into<BigUint> for U256 {
    fn into(self) -> BigUint {
        BigUint::from_slice(self.0 .0.as_byte_slice().as_slice_of::<u32>().unwrap())
    }
}
