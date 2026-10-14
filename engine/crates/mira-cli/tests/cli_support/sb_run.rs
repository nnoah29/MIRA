//! Lancement des commandes du CLI sur le banc des scénarios, avec ou sans synchro de fond.

use std::time::Duration;

use serde_json::Value;
use tokio::task::JoinHandle;

use super::sb_world::{SB_PASSWORD, SbWorld};
use super::{CliResult, account_id_of, parse_json, run_with};

impl SbWorld {
    /// Lance une commande (au plus 60 secondes) avec l'entrée standard donnée, sans synchro de fond.
    pub async fn run_stdin(&self, stdin: &str, arguments: &[&str]) -> CliResult {
        let mut deps = self.deps(stdin, false);
        let limit = Duration::from_secs(60);
        tokio::time::timeout(limit, run_with(&mut deps, arguments))
            .await
            .unwrap()
    }

    /// Lance une commande avec une entrée standard vide, sans synchro de fond.
    pub async fn run(&self, arguments: &[&str]) -> CliResult {
        self.run_stdin("", arguments).await
    }

    /// Lance une commande avec `--json`, exige le code 0 et rend la réponse du contrat.
    pub async fn run_ok(&self, arguments: &[&str]) -> Value {
        let mut full: Vec<&str> = arguments.to_vec();
        full.push("--json");
        let result = self.run(&full).await;
        assert_eq!(result.code, 0, "{arguments:?} : {}", result.stderr);
        parse_json(&result)["data"].clone()
    }

    /// Lance une commande de synchro (synchro de fond permise, au plus 60 secondes) et rend son résultat.
    pub async fn run_sync(&self, arguments: &[&str]) -> CliResult {
        let mut deps = self.deps("", true);
        let limit = Duration::from_secs(60);
        tokio::time::timeout(limit, run_with(&mut deps, arguments))
            .await
            .unwrap()
    }

    /// Ajoute le compte (mot de passe sur l'entrée standard, d'application pour gmail.com) et rend son identifiant.
    pub async fn add_account(&self, user: &str) -> String {
        let stdin = format!("{SB_PASSWORD}\n");
        let method = if user.ends_with("@gmail.com") {
            "app-password"
        } else {
            "password"
        };
        let mut arguments = vec!["account", "add", "--address", user];
        arguments.extend(["--auth", method, "--password-stdin", "--json"]);
        let added = self.run_stdin(&stdin, &arguments).await;
        assert_eq!(added.code, 0, "{}", added.stderr);
        account_id_of(&added)
    }

    /// Fait avancer l'horloge de 11 s toutes les 100 ms réelles, jusqu'à l'arrêt de la tâche.
    pub fn pump_clock(&self) -> JoinHandle<()> {
        let clock = std::sync::Arc::clone(&self.clock);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(100)).await;
                clock.advance_millis(11_000);
            }
        })
    }
}
