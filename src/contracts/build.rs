use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

use ethers::contract::MultiAbigen;
use ethers::solc::{self, artifacts::Severity, Project};
use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
    static ref CONTRACTS_DIR: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts")
        .canonicalize()
        .unwrap();
    static ref PROJECTS_FILE: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("projects.toml")
        .canonicalize()
        .unwrap();
}

#[derive(Deserialize)]
struct ProjectPathsConfig {
    root: PathBuf,
    sources: Option<PathBuf>,
    #[serde(default)]
    libraries: Vec<PathBuf>,
    artifacts: Option<PathBuf>,
    cache: Option<PathBuf>,
}

impl From<ProjectPathsConfig> for solc::ProjectPathsConfig {
    fn from(value: ProjectPathsConfig) -> Self {
        let ProjectPathsConfig {
            root,
            sources,
            libraries,
            artifacts,
            cache,
        } = value;

        let root = CONTRACTS_DIR.join(root);

        let mut b =
            solc::ProjectPathsConfig::builder().libs(libraries.into_iter().map(|l| root.join(l)));

        if let Some(sources) = sources {
            b = b.sources(root.join(sources));
        }
        if let Some(artifacts) = artifacts {
            b = b.artifacts(root.join(artifacts));
        }
        if let Some(cache) = cache {
            b = b.cache(root.join(cache));
        }

        b.build_with_root(root)
    }
}

fn main() {
    let projects: HashMap<String, ProjectPathsConfig> = toml::from_str(
        fs::read_to_string(PROJECTS_FILE.as_path())
            .unwrap()
            .as_str(),
    )
    .unwrap();
    println!("cargo:rerun-if-changed={}", PROJECTS_FILE.display());

    for (feature, cfg) in projects {
        let module_path = feature.replace("-", "_");
        {
            let feature_env = format!("CARGO_FEATURE_{}", module_path.to_ascii_uppercase());
            if env::var(&feature_env).is_err() {
                println!("cargo:rerun-if-env-changed={feature_env}");
                continue;
            }
        }

        let project = Project::builder()
            .paths(cfg.into())
            .offline()
            .build()
            .unwrap();
        {
            let out = project.compile().unwrap().output();
            project.rerun_if_sources_changed();

            let diagnostics = out.diagnostics(&[], Severity::Info);
            if diagnostics.has_error() {
                panic!("{}", diagnostics);
            }
            if diagnostics.has_warning() {
                println!("cargo:warning={}", diagnostics);
            }
        }

        MultiAbigen::from_json_files(project.artifacts_path())
            .unwrap()
            .build()
            .unwrap()
            .write_to_module(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("./src")
                    .join(module_path),
                false,
            )
            .unwrap();

        println!(
            "cargo:rerun-if-changed={}",
            project.artifacts_path().display()
        );
    }
}
