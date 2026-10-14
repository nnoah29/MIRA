//! Exécution des commandes dans un monde de scénario ; chaque sortie est gardée dans le journal.

use std::sync::Arc;
use std::time::Duration;

use mira_assembly::ports::{Clock, SystemClock};
use mira_cli::CliDeps;

use serde_json::Value;

use super::sa_world::SaWorld;
use super::sync_bench::quick_sync;
use super::{CliResult, parse_json, run_with};

impl SaWorld {
    /// Comme `deps`, avec l'horloge réelle et la synchro de fond permise.
    pub fn sync_deps(&self, stdin: &str) -> CliDeps {
        let mut deps = self.deps(stdin);
        deps.clock = Arc::new(SystemClock) as Arc<dyn Clock>;
        deps.network.background_sync = quick_sync();
        deps
    }

    /// Garde stdout et stderr d'une commande dans le journal du monde.
    fn record(&self, result: &CliResult) {
        let mut log = self.transcript.lock().unwrap();
        log.push(result.stdout.clone());
        log.push(result.stderr.clone());
    }

    /// Lance une commande avec cette entrée standard (au plus 60 secondes), synchro de fond coupée.
    pub async fn run_stdin(&self, stdin: &str, arguments: &[&str]) -> CliResult {
        let mut deps = self.deps(stdin);
        let limit = Duration::from_secs(60);
        let result = tokio::time::timeout(limit, run_with(&mut deps, arguments))
            .await
            .unwrap();
        self.record(&result);
        result
    }

    /// Lance une commande avec la synchro de fond permise (au plus 60 secondes).
    pub async fn run_sync(&self, arguments: &[&str]) -> CliResult {
        let mut deps = self.sync_deps("");
        let limit = Duration::from_secs(60);
        let result = tokio::time::timeout(limit, run_with(&mut deps, arguments))
            .await
            .unwrap();
        self.record(&result);
        result
    }

    /// Lance une commande `--json` (entrée vide), exige le code 0 et rend le champ `data` de l'enveloppe.
    pub async fn run_ok(&self, arguments: &[&str]) -> Value {
        let mut full: Vec<&str> = arguments.to_vec();
        full.push("--json");
        let result = self.run_stdin("", &full).await;
        assert_eq!(result.code, 0, "{arguments:?} : {}", result.stderr);
        parse_json(&result)["data"].clone()
    }

    /// Lance une commande de synchro `--json`, exige le code 0 et rend le champ `data` de l'enveloppe.
    pub async fn run_sync_ok(&self, arguments: &[&str]) -> Value {
        let mut full: Vec<&str> = arguments.to_vec();
        full.push("--json");
        let result = self.run_sync(&full).await;
        assert_eq!(result.code, 0, "{arguments:?} : {}", result.stderr);
        parse_json(&result)["data"].clone()
    }
}
