#![feature(iterator_try_collect)]

macro_rules! check_mod {
    ($feature:literal => $module:ident) => {
        #[cfg(feature = $feature)]
        pub mod $module;
    };
    ($feature:literal => $module:ident, $($features:literal => $modules:ident),+) => {
        check_mod!($feature => $module);
        check_mod!($($features => $modules),+);
    };
}

check_mod! {
    "multicall" => multicall_utils,
    "pancake_swap" => pancake_swap,
    "pancake_toaster" => pancake_toaster,
    "uniswap_core_v2" => uniswap_core_v2
}
