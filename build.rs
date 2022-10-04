use std::path::Path;

use ethers_contract::MultiAbigen;

const CONTRACTS_ABI_DIR: &str = "./contracts";

fn main() {
    MultiAbigen::from_json_files(CONTRACTS_ABI_DIR)
        .unwrap()
        .build()
        .unwrap()
        .write_to_module(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/contracts"),
            false,
        )
        .unwrap();
    println!("cargo:rerun-if-changed={CONTRACTS_ABI_DIR}");
}
