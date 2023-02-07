use std::path::Path;

use bindgen::*;

fn main() {
    compile_and_generate(
        {
            let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts");
            ProjectPathsConfig::builder()
                .sources(root.join("src/pancake_toaster"))
                .lib(root.join("lib"))
                .artifacts(root.join("out"))
                .cache(root.join("cache"))
                .root(root)
                .build()
                .unwrap()
        },
        Path::new(env!("CARGO_MANIFEST_DIR")).join("./src/contracts"),
    );
}
