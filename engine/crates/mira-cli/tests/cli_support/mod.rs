//! Exécution du CLI en processus, avec dépendances injectées (faux trousseau) et capture des flux.

pub mod bench_tools;
pub mod fake_oauth_provider;
pub mod full_bench;
pub mod json_views;
pub mod mirror_reads;
pub mod oauth_bench;
pub mod raw_mail;
pub mod seed;
pub mod seed_fixtures;
pub mod server;
pub mod shared_buffer;
pub mod sync_bench;

use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

use mira_assembly::accounts::NetworkPolicy;
use mira_assembly::auth::OAuthClientConfig;
use mira_assembly::build_registry;
use mira_assembly::mira_vocabulary::Timestamp;
use mira_assembly::ports::{ManualClock, MemorySecretStore, SequentialIds};
use mira_assembly::profile::ProfileEnvironment;
use mira_cli::{CliDeps, CliIo, run};

pub mod sa_flows;
pub mod sa_leak;
pub mod sa_mail;
pub mod sa_s6_reading;
pub mod sa_s7_moves;
pub mod sa_world;
pub mod sa_world_runs;

/// Résultat d'une exécution : code de sortie, stdout et stderr.
#[derive(Debug)]
pub struct CliResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Dépendances de test : données sous `home`, trousseau en mémoire, `stdin` fourni.
pub fn test_deps(
    home: &Path,
    secrets: Arc<MemorySecretStore>,
    network: NetworkPolicy,
    stdin: &str,
) -> CliDeps {
    CliDeps {
        environment: ProfileEnvironment {
            mira_home: Some(home.to_path_buf()),
            xdg_data_home: None,
            home: None,
        },
        registry: build_registry(),
        secrets,
        clock: Arc::new(ManualClock::new(Timestamp::from_millis(1_700_000_000_000))),
        ids: Arc::new(SequentialIds::new()),
        oauth: OAuthClientConfig::default(),
        network,
        stdin: Box::new(Cursor::new(stdin.as_bytes().to_vec())),
    }
}

/// Lance `mira` avec ces arguments (sans le nom du programme) et capture ses flux.
pub async fn run_with(deps: &mut CliDeps, arguments: &[&str]) -> CliResult {
    let mut args = vec!["mira".to_string()];
    args.extend(arguments.iter().map(ToString::to_string));
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(args, &mut CliIo::new(&mut out, &mut err), deps).await;
    CliResult {
        code,
        stdout: String::from_utf8_lossy(&out).into_owned(),
        stderr: String::from_utf8_lossy(&err).into_owned(),
    }
}

/// Analyse la sortie standard comme un document JSON.
pub fn parse_json(result: &CliResult) -> serde_json::Value {
    serde_json::from_str(&result.stdout).unwrap()
}

/// Identifiant du compte rendu par `account add --json`.
pub fn account_id_of(result: &CliResult) -> String {
    let document = parse_json(result);
    document["data"]["data"]["id"].as_str().unwrap().to_string()
}

/// Version synchrone sans réseau : profil jetable, trousseau vide, entrée standard vide.
pub fn run_cli(arguments: &[&str]) -> CliResult {
    let home = tempfile::tempdir().unwrap();
    let secrets = Arc::new(MemorySecretStore::new());
    let mut deps = test_deps(home.path(), secrets, NetworkPolicy::default(), "");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(run_with(&mut deps, arguments))
}
