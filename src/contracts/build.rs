#![feature(iterator_try_collect)]

use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::{env, fs};

use ethers::contract::{Abigen, MultiAbigen};
use ethers::solc::{
    self,
    artifacts::Severity,
    remappings::{RelativeRemapping, RelativeRemappingPathBuf},
    Project,
};
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
    let out_dir: PathBuf = env::var_os("OUT_DIR").expect("OUT_DIR is not set").into();
    let projects: HashMap<String, ProjectConfig> =
        toml::from_str(&fs::read_to_string(PROJECTS_FILE.as_path()).unwrap()).unwrap();
    println!("cargo:rerun-if-changed={}", PROJECTS_FILE.display());

    for (
        feature,
        ProjectConfig {
            mod_name,
            paths: cfg,
        },
    ) in projects
    {
        let mod_name = mod_name.unwrap_or_else(|| feature.replace("-", "_").into());
        {
            if env::var(format!("CARGO_FEATURE_{}", mod_name.to_uppercase())).is_err() {
                continue;
            }
        }

        let cfg: solc::ProjectPathsConfig = cfg.into();
        cfg.create_all().unwrap();

        let project = Project::builder()
            .paths(cfg)
            .offline()
            .no_auto_detect()
            .build()
            .unwrap();
        for p in Some(&project.paths.sources)
            .into_iter()
            .chain(project.paths.libraries.iter())
        {
            println!("cargo:rerun-if-changed={}", p.display());
        }

        let compiled = project.compile().unwrap();

        let artifact_files: Vec<PathBuf> = compiled
            .compiled_artifacts()
            .artifact_files()
            .chain(compiled.cached_artifacts().artifact_files())
            .map(|a| &a.file)
            .cloned()
            .collect();

        let output = compiled.output();
        let diagnostics = output.diagnostics(&[], Severity::Error);
        if diagnostics.has_error() {
            panic!("{}", diagnostics);
        }
        let diagnostics = output.diagnostics(&[], Severity::Warning);
        if diagnostics.has_warning() {
            println!("cargo:warning={}", diagnostics);
        }

        println!(
            "cargo:rerun-if-changed={}",
            project.artifacts_path().display()
        );

        artifact_files
            .iter()
            .map(|a| Abigen::from_file(a).map(|a| a.rustfmt(true)))
            .try_collect::<MultiAbigen>()
            .unwrap()
            .build()
            .unwrap()
            .write_to_module(
                out_dir.join(mod_name),
                true,
            )
            .unwrap();
    }
}

#[derive(Debug, Deserialize)]
struct ProjectConfig {
    #[serde(rename = "mod")]
    mod_name: Option<String>,
    #[serde(flatten)]
    paths: ProjectPathsConfig,
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
            ..
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
