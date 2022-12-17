#![feature(iterator_try_collect)]
use std::path::Path;

use ethers_contract::MultiAbigen;
use ethers_solc::{Project, ProjectPathsConfig};

const CONTRACTS_DIR: &str = "./contracts";
const CONTRACTS_BINDINGS_DIR: &str = "./contracts";

fn main() {
    let project = Project::builder()
        .paths({
            let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(CONTRACTS_DIR);
            ProjectPathsConfig::builder()
                .sources(root.join("src"))
                .artifacts(root.join("out"))
                .lib(root.join("lib"))
                .root(root)
                .build()
                .unwrap()
        })
        .offline()
        .build()
        .unwrap();

    let _ = project.compile().unwrap();
    project.rerun_if_sources_changed();

    MultiAbigen::from_json_files(project.artifacts_path())
        .unwrap()
        .build()
        .unwrap()
        .write_to_module(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join(CONTRACTS_BINDINGS_DIR),
            false,
        )
        .unwrap();
}
