use std::path::Path;

use ethers_contract::MultiAbigen;

fn main() {
    MultiAbigen::from_json_files(Path::new(env!("CARGO_MANIFEST_DIR")).join("contracts"))
        .unwrap()
        .build()
        .unwrap()
        .write_to_module(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/contracts"),
            false,
        )
        .unwrap();
}
