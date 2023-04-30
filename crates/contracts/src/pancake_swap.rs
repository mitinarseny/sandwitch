use crate::prelude::*;

pub mod router {
    use super::*;

    tracked_abigen!(
        PancakeRouter,
        "contracts/out/IPancakeRouter02.sol/IPancakeRouter02.json"
    );
}

pub mod pair {
    use super::*;

    tracked_abigen!(
        PancakePair,
        "contracts/out/IPancakePair.sol/IPancakePair.json"
    );
}
