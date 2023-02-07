use std::path::Path;

use bindgen::*;

fn main() {
    compile_and_generate(
        {
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/lib/pancake-smart-contracts/projects/exchange-protocol");
            ProjectPathsConfig::builder()
                .sources(root.join("contracts/interfaces"))
                .lib(root.join("contracts/libraries"))
                .artifacts(root.join("artifacts"))
                .cache(root.join("cache"))
                .tests(root.join("test"))
                .root(root)
                .build()
                .unwrap()
        },
        Path::new(env!("CARGO_MANIFEST_DIR")).join("./src/contracts"),
    );
}
