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
    "pancake_swap" => pancake_swap,
    "uniswap_core_v2" => uniswap_core_v2,
    "pancake_toaster" => pancake_toaster
}
