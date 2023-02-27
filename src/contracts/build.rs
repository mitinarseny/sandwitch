use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

use ethers::contract::MultiAbigen;
use ethers::solc::remappings::{RelativeRemapping, RelativeRemappingPathBuf};
use ethers::solc::{self, artifacts::Severity, Project};
use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
    static ref CONTRACTS_DIR: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts")
        .canonicalize()
        .unwrap();
    static ref PROJECTS_FILE: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("./projects.toml")
        .canonicalize()
        .unwrap();
}

fn main() {
    let projects: HashMap<String, ProjectPathsConfig> =
        toml::from_str(&fs::read_to_string(PROJECTS_FILE.as_path()).unwrap()).unwrap();
    println!("cargo:rerun-if-changed={}", PROJECTS_FILE.display());

    for (feature, cfg) in projects {
        let module_path = feature.replace("-", "_");
        {
            if env::var(format!(
                "CARGO_FEATURE_{}",
                module_path.to_ascii_uppercase()
            ))
            .is_err()
            {
                continue;
            }
        }

        let project = Project::builder()
            .paths(cfg.into())
            .offline()
            .build()
            .unwrap();
        let paths = Some(&project.paths.sources)
            .into_iter()
            .chain(project.paths.libraries.iter());
        // TODO: add other paths

        for p in paths {
            println!("cargo:rerun-if-changed={}", p.display());
        }

        {
            let out = project.compile().unwrap().output();
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

#[derive(Debug, Deserialize)]
struct ProjectPathsConfig {
    root: PathBuf,
    sources: Option<PathBuf>,
    #[serde(default)]
    libraries: Vec<PathBuf>,
    #[serde(default)]
    remappings: HashMap<String, PathBuf>,
    artifacts: Option<PathBuf>,
    cache: Option<PathBuf>,
}

impl From<ProjectPathsConfig> for solc::ProjectPathsConfig {
    fn from(value: ProjectPathsConfig) -> Self {
        let ProjectPathsConfig {
            root,
            sources,
            libraries,
            remappings,
            artifacts,
            cache,
        } = value;

        let root = CONTRACTS_DIR.join(root);

        let mut b = solc::ProjectPathsConfig::builder()
            .libs(libraries.into_iter().map(|l| root.join(l)))
            .remappings(remappings.into_iter().map(|(name, path)| {
                RelativeRemapping {
                    name,
                    path: RelativeRemappingPathBuf { parent: None, path },
                }
                .to_remapping(CONTRACTS_DIR.clone())
            }));

        if let Some(sources) = sources {
            b = b.sources(root.join(sources));
        }
        if let Some(artifacts) = artifacts {
            b = b.artifacts(root.join(artifacts));
        }
        if let Some(cache) = cache {
            b = b.cache(root.join(cache));
        }

        let pp = b.build_with_root(root);
        pp
    }
}
