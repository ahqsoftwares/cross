use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::errors::*;
use crate::extensions::CommandExt;

const DOCKER: &str = "docker";
const PODMAN: &str = "podman";

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EngineType {
    Docker,
    Podman,
    PodmanRemote,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Engine {
    pub kind: EngineType,
    pub path: PathBuf,
}

impl Engine {
    pub fn new(verbose: bool) -> Result<Engine> {
        let path = get_container_engine()
            .map_err(|_| eyre::eyre!("no container engine found"))
            .with_suggestion(|| "is docker or podman installed?")?;
        Self::from_path(path, verbose)
    }

    pub fn from_path(path: PathBuf, verbose: bool) -> Result<Engine> {
        let kind = get_engine_type(&path, verbose)?;
        Ok(Engine { path, kind })
    }
}

// determine if the container engine is docker. this fixes issues with
// any aliases (#530), and doesn't fail if an executable suffix exists.
fn get_engine_type(ce: &Path, verbose: bool) -> Result<EngineType> {
    let stdout = Command::new(ce)
        .arg("--help")
        .run_and_get_stdout(verbose)?
        .to_lowercase();

    if stdout.contains("podman-remote") {
        Ok(EngineType::PodmanRemote)
    } else if stdout.contains("podman") {
        Ok(EngineType::Podman)
    } else if stdout.contains("docker") && !stdout.contains("emulate") {
        Ok(EngineType::Docker)
    } else {
        Ok(EngineType::Other)
    }
}

pub fn get_container_engine() -> Result<PathBuf, which::Error> {
    if let Ok(ce) = env::var("CROSS_CONTAINER_ENGINE") {
        which::which(ce)
    } else {
        which::which(DOCKER).or_else(|_| which::which(PODMAN))
    }
}