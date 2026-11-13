#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Construction de la fixture : fiche du runtime pointant vers les faux hébergeurs, moteur ouvert dessus.

use std::sync::Arc;

use mira_core::ai::AiProviderRegistry;
use mira_core::api::Engine;
use mira_core::ports::FixedResources;
use mira_core::profile::ProfilePaths;
use mira_vocabulary::{AiLocalRuntime, AiProvider, AiProviderDescriptor, CONTRACT_VERSION};

use super::OpsFixture;
use crate::local_support::archives::{Entry, tar_gz};
use crate::local_support::download::{content, sha_hex};
use crate::local_support::file_host::FileHost;
use crate::local_support::runtime_fake::fake_args;
use crate::local_support::{descriptor, test_build, test_runtime};
use crate::support_api::config;

const FAKE: &str = include_str!("../local_support/fake_server.py");

/// Fournisseur de test qui rend une fiche donnée
#[derive(Debug)]
struct Provider(AiProviderDescriptor);

impl AiProvider for Provider {
    /// Rend la fiche portée par le fournisseur
    fn descriptor(&self) -> AiProviderDescriptor {
        self.0.clone()
    }
}

/// Fiche du runtime : serveur et deux modèles pris chez les faux hébergeurs
fn runtime_for(
    hosts: [&FileHost; 3],
    archive: &[u8],
    bodies: [&[u8]; 2],
    platforms: &[&str],
    log: &std::path::Path,
) -> AiLocalRuntime {
    let mut runtime = test_runtime();
    runtime.server = platforms
        .iter()
        .map(|p| {
            let mut build = test_build(p);
            (build.url, build.sha256, build.size) =
                (hosts[0].url(), sha_hex(archive), archive.len() as u64);
            build
        })
        .collect();
    for (model, (host, body)) in runtime
        .models
        .iter_mut()
        .zip([(hosts[1], bodies[0]), (hosts[2], bodies[1])])
    {
        (model.url, model.sha256, model.size) = (host.url(), sha_hex(body), body.len() as u64);
    }
    runtime.models[0].ram_recommended_mib = 1000;
    runtime.models[1].ram_recommended_mib = 9000;
    runtime.server_args = fake_args(log, &[]);
    runtime.start_timeout_seconds = 5;
    runtime
}

/// Démarre les hébergeurs et le moteur
pub(super) async fn build(memory_mib: Option<u64>, platforms: &[&str]) -> OpsFixture {
    let archive = tar_gz(&[Entry::file("bin/model-server", FAKE.as_bytes())]);
    let (mini, big) = (content(40_000), content(60_000));
    let server_host = FileHost::start(archive.clone()).await;
    let mini_host = FileHost::start(mini.clone()).await;
    let big_host = FileHost::start(big.clone()).await;
    let home = tempfile::tempdir().unwrap();
    let log = home.path().join("log.jsonl");
    let hosts = [&server_host, &mini_host, &big_host];
    let runtime = runtime_for(hosts, &archive, [&mini, &big], platforms, &log);
    let (engine, root) = open_engine(home.path(), runtime, memory_mib);
    OpsFixture {
        home,
        engine,
        root,
        log,
        server_host,
        mini_host,
        big_host,
    }
}

/// Ouvre le moteur sur ce runtime et rend aussi le dossier `local-ai` du profil
fn open_engine(
    home: &std::path::Path,
    runtime: AiLocalRuntime,
    memory_mib: Option<u64>,
) -> (Arc<Engine>, std::path::PathBuf) {
    let mut config = config(home);
    config.resources = Arc::new(FixedResources::new(memory_mib, Some(u64::MAX)));
    let mut registry = AiProviderRegistry::new();
    registry.register(Arc::new(Provider(descriptor(
        "local-test",
        false,
        Some(runtime),
    ))));
    config.ai_registry = registry;
    let paths = ProfilePaths::resolve(&config.profile, &config.environment).unwrap();
    let engine = Arc::new(Engine::open(config, CONTRACT_VERSION).unwrap());
    (engine, paths.root.join("local-ai"))
}
