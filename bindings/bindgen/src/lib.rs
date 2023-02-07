use std::path::Path;

use ethers::contract::MultiAbigen;
pub use ethers::solc::ProjectPathsConfig;
use ethers::solc::{artifacts::Severity, Project};

pub fn compile_and_generate(cfg: ProjectPathsConfig, module_path: impl AsRef<Path>) {
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
