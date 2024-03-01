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

pub mod factory {
    use super::*;

    tracked_abigen!(
        PancakeFactory,
        "contracts/out/IPancakeFactory.sol/IPancakeFactory.json"
    );
}
