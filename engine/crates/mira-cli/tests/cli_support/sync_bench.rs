//! Banc de synchro : commandes lancées avec la synchro de fond active (horloge réelle, délais courts).

use std::sync::Arc;
use std::time::Duration;

use mira_assembly::ports::{Clock, SystemClock};
use mira_assembly::sync::{BackgroundSync, BackoffPolicy, SupervisorConfig};
use mira_cli::CliDeps;
use serde_json::Value;

use super::full_bench::FullBench;
use super::{CliResult, parse_json, run_with};

/// Délais courts ; le relevé périodique est très espacé pour que seules les demandes agissent.
pub fn quick_sync() -> BackgroundSync {
    BackgroundSync::On(SupervisorConfig {
        backoff: BackoffPolicy {
            initial: Duration::from_millis(50),
            max: Duration::from_millis(200),
        },
        idle_renewal: Duration::from_secs(30),
        poll_interval: Some(Duration::from_secs(600)),
    })
}

impl FullBench {
    /// Dépendances dont la synchro de fond est permise (le CLI ne l'active que pour les commandes de synchro).
    pub fn sync_deps(&self, stdin: &str) -> CliDeps {
        let mut deps = self.deps(stdin);
        deps.clock = Arc::new(SystemClock) as Arc<dyn Clock>;
        deps.network.background_sync = quick_sync();
        deps
    }

    /// Lance une commande avec la synchro permise (au plus 60 secondes).
    pub async fn run_sync(&self, arguments: &[&str]) -> CliResult {
        let mut deps = self.sync_deps("");
        let limit = Duration::from_secs(60);
        tokio::time::timeout(limit, run_with(&mut deps, arguments))
            .await
            .unwrap()
    }

    /// Lance une commande de synchro avec `--json`, exige le code 0 et rend la réponse du contrat.
    pub async fn run_sync_ok(&self, arguments: &[&str]) -> Value {
        let mut full: Vec<&str> = arguments.to_vec();
        full.push("--json");
        let result = self.run_sync(&full).await;
        assert_eq!(result.code, 0, "{arguments:?} : {}", result.stderr);
        parse_json(&result)["data"].clone()
    }
}
