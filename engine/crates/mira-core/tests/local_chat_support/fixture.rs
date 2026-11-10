#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Moteur ouvert avec un fournisseur local de test : faux serveur posé dans l'arborescence du profil.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use mira_core::ai::AiProviderRegistry;
use mira_core::api::Engine;
use mira_core::profile::ProfilePaths;
use mira_core::storage::Database;
use mira_vocabulary::{
    AiConnection, AiConnectionId, AiProvider, AiProviderDescriptor, AiProviderId, CONTRACT_VERSION,
    MaskingSettings,
};
use tempfile::TempDir;

use crate::local_support::runtime_fake::fake_args;
use crate::local_support::{descriptor, test_runtime};
use crate::support_api::config;

const FAKE: &str = include_str!("../local_support/fake_llama_server.py");

/// Fournisseur de test qui rend une fiche donnée
#[derive(Debug)]
struct LocalTestProvider(AiProviderDescriptor);

impl AiProvider for LocalTestProvider {
    /// Rend la fiche portée par le fournisseur
    fn descriptor(&self) -> AiProviderDescriptor {
        self.0.clone()
    }
}

/// Moteur de test dont le profil contient déjà serveur et modèles factices
pub struct LocalFixture {
    pub home: TempDir,
    pub engine: Arc<Engine>,
    pub db: Database,
    pub root: PathBuf,
    pub log: PathBuf,
}

impl LocalFixture {
    /// Ouvre le moteur, pose `llama-server` (le faux) et les fichiers des modèles `mini` et `big`
    pub fn start(idle_stop_seconds: u64) -> Self {
        let home = tempfile::tempdir().unwrap();
        let mut config = config(home.path());
        let paths = ProfilePaths::resolve(&config.profile, &config.environment).unwrap();
        let log = home.path().join("log.jsonl");
        let mut runtime = test_runtime();
        runtime.server_args = fake_args(&log, &[]);
        runtime.idle_stop_seconds = idle_stop_seconds;
        runtime.start_timeout_seconds = 5;
        place_files(&paths.root, &runtime.server[0].sha256);
        let provider = LocalTestProvider(descriptor("local-test", false, Some(runtime)));
        let mut registry = AiProviderRegistry::new();
        registry.register(Arc::new(provider));
        config.ai_registry = registry;
        let engine = Arc::new(Engine::open(config, CONTRACT_VERSION).unwrap());
        let db = Database::open(&paths.database).unwrap();
        Self {
            home,
            engine,
            db,
            root: paths.root.join("local-ai"),
            log,
        }
    }

    /// Insère la connexion `local-1` sur ce fournisseur avec ce modèle par défaut
    pub fn add_connection(&self, default_model: Option<&str>) -> AiConnection {
        let connection = AiConnection {
            id: AiConnectionId::from_raw("local-1"),
            provider: AiProviderId("local-test".to_string()),
            label: "Modèle local".to_string(),
            base_url: None,
            default_model: default_model.map(str::to_string),
            is_default: false,
            has_key: false,
            consent_at: None,
            models: None,
            allowed_kinds: Vec::new(),
            masking: MaskingSettings::local_default(),
        };
        self.db
            .write(|tx| tx.insert_ai_connection(&connection, 1))
            .unwrap();
        connection
    }

    /// Lancements enregistrés par le faux serveur
    pub fn starts(&self) -> usize {
        self.entries("start").len()
    }

    /// Entrées du journal du faux serveur pour cet événement
    pub fn entries(&self, event: &str) -> Vec<serde_json::Value> {
        let text = std::fs::read_to_string(&self.log).unwrap_or_default();
        let all = text
            .lines()
            .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap());
        all.filter(|v| v["event"] == event).collect()
    }
}

/// Pose le faux serveur et les deux modèles dans `<profil>/local-ai/`
fn place_files(profile_root: &Path, sha256: &str) {
    let base = profile_root.join("local-ai");
    let server = base.join("server").join(&sha256[..12]);
    std::fs::create_dir_all(&server).unwrap();
    let exe = server.join("llama-server");
    std::fs::write(&exe, FAKE).unwrap();
    std::fs::set_permissions(&exe, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
    std::fs::create_dir_all(base.join("models")).unwrap();
    for name in ["mini.bin", "big.bin"] {
        std::fs::write(base.join("models").join(name), b"GGUF").unwrap();
    }
}
