use std::env;
use std::path::Path;

use ethers::contract::MultiAbigen;
use ethers::solc::{artifacts::Severity, Project, ProjectPathsConfig};

fn main() {
    let contracts_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts")
        .canonicalize()
        .unwrap();

    check_compile_and_generate("pancake_swap", || {
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

    check_compile_and_generate("pancake_toaster", || {
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

fn check_compile_and_generate(feature: &str, cfg: impl FnOnce() -> ProjectPathsConfig) {
    let module_path = feature.replace("-", "_");
    if !env::var(format!(
        "CARGO_FEATURE_{}",
        module_path.to_ascii_uppercase()
    ))
    .is_ok()
    {
        return;
    }

    compile_and_generate(Path::new("./src").join(module_path), cfg())
}

fn compile_and_generate(module_path: impl AsRef<Path>, cfg: ProjectPathsConfig) {
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
