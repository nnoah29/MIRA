#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Outils des tests du gestionnaire d'exécution : faux serveur Python, descripteur et journal de lancements.

use std::path::{Path, PathBuf};

use mira_core::ai::local::RuntimeContext;
use mira_core::ports::FixedResources;
use mira_vocabulary::AiLocalRuntime;

use super::runtime_fake::{fake_args, install_fake};
use super::test_runtime;

/// Dossier de test avec le faux serveur exécutable, un modèle factice et un journal
pub struct RuntimeFixture {
    pub dir: tempfile::TempDir,
    pub runtime: AiLocalRuntime,
    pub resources: FixedResources,
}

impl RuntimeFixture {
    /// Prépare le dossier : faux serveur 0755, fichiers de modèle `mini` et `big`, journal vide
    pub fn new(extra_args: &[&str]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        install_fake(dir.path());
        let mut runtime = test_runtime();
        runtime.server_args = fake_args(&dir.path().join("log.jsonl"), extra_args);
        runtime.start_timeout_seconds = 5;
        Self {
            dir,
            runtime,
            resources: FixedResources::new(Some(16_000), Some(u64::MAX)),
        }
    }

    /// Contexte d'exécution sur ce dossier
    pub fn context(&self) -> RuntimeContext<'_> {
        RuntimeContext {
            runtime: &self.runtime,
            executable: self.dir.path().join("fake-server"),
            run_dir: self.dir.path().join("run"),
            models_dir: self.dir.path().join("models"),
            resources: &self.resources,
        }
    }

    /// Lignes du journal du faux serveur, une valeur JSON par ligne
    pub fn log(&self) -> Vec<serde_json::Value> {
        let text = std::fs::read_to_string(self.dir.path().join("log.jsonl")).unwrap_or_default();
        text.lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    /// Lancements enregistrés par le faux serveur
    pub fn starts(&self) -> Vec<serde_json::Value> {
        self.log()
            .into_iter()
            .filter(|l| l["event"] == "start")
            .collect()
    }

    /// Dit si le processus de ce `pid` existe encore
    pub fn alive(pid: u64) -> bool {
        Path::new(&format!("/proc/{pid}")).exists()
    }

    /// Chemin du dossier d'exécution
    pub fn run_dir(&self) -> PathBuf {
        self.dir.path().join("run")
    }
}
