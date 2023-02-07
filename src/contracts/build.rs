use std::collections::HashMap;
use std::env;
use std::path::Path;

use ethers::contract::MultiAbigen;
use ethers::solc::{artifacts::Severity, Project, ProjectPathsConfig};

fn main() {
    let contracts_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts")
        .canonicalize()
        .unwrap();

    let mut projects = HashMap::<&str, ProjectPathsConfig>::new();

    if env::var("CARGO_FEATURE_PANCAKE_SWAP").is_ok() {
        projects.insert("pancake_swap", {
            let root = contracts_dir.join("lib/pancake-smart-contracts/projects/exchange-protocol");
            ProjectPathsConfig::builder()
                .sources(root.join("contracts/interfaces"))
                .lib(root.join("contracts/libraries"))
                .artifacts(root.join("artifacts"))
                .cache(root.join("cache"))
                .tests(root.join("test"))
                .root(root)
                .build()
                .unwrap()
        });
    }

    if env::var("CARGO_FEATURE_PANCAKE_TOASTER").is_ok() {
        projects.insert("pancake_toaster", {
            let root = contracts_dir.clone();
            ProjectPathsConfig::builder()
                .sources(root.join("src"))
                .lib(root.join("lib"))
                .artifacts(root.join("out"))
                .cache(root.join("cache"))
                .root(root)
                .build()
                .unwrap()
        });
    }

    for (module_path, cfg) in projects {
        compile_and_generate(cfg, Path::new("./src").join(module_path));
    }
}

fn compile_and_generate(cfg: ProjectPathsConfig, module_path: impl AsRef<Path>) {
    let project = Project::builder().paths(cfg).offline().build().unwrap();

    let out = project.compile().unwrap().output();
    project.rerun_if_sources_changed();

    let diagnostics = out.diagnostics(&[], Severity::Info);
    if diagnostics.has_error() {
        panic!("{}", diagnostics);
    }
    if diagnostics.has_warning() {
        println!("cargo:warning={}", diagnostics);
    }

    MultiAbigen::from_json_files(project.artifacts_path())
        .unwrap()
        .build()
        .unwrap()
        .write_to_module(module_path, false)
        .unwrap();

    println!(
        "cargo:rerun-if-changed={}",
        project.artifacts_path().display()
    );
}
