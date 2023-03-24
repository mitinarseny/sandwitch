#![feature(iterator_try_collect)]

pub(crate) mod utils;

macro_rules! include_abigen {
    ($name:expr) => {
        include!(concat!(env!("OUT_DIR"), "/", $name, "/mod.rs"));
    }
}

macro_rules! mod_abigen {
    ($vis:vis mod $module:ident => $name:expr) => {
        $vis mod $module {
            include_abigen!($name);
        }
    };
    ($vis:vis mod $module:ident) => {
        mod_abigen!($vis mod $module => stringify!($module));
    }
}

macro_rules! if_feature_mod_abigen {
    ($feature:literal => $vis:vis mod $module:ident) => {
        #[cfg(feature = $feature)]
        mod_abigen!($vis mod $module => $feature);
    };
}

#[cfg(feature = "multicall")]
pub mod multicall;

if_feature_mod_abigen!("pancake_swap" => pub mod pancake_swap);