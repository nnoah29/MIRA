//! Outils du banc complet : ajout du compte, miroir local, horloge qui avance pendant une attente.

use std::time::Duration;

use mira_assembly::mira_vocabulary::ProfileId;
use mira_assembly::profile::ProfilePaths;
use mira_assembly::storage::Database;
use serde_json::Value;
use tokio::task::JoinHandle;

use super::full_bench::{FullBench, PASSWORD, USER};
use super::raw_mail::{plain, with_attachment};
use super::seed::{seed_bodies, seed_mirror, seed_threads};

impl FullBench {
    /// Banc dont le compte est ajouté et le miroir garni (boîtes, messages, corps, fils).
    pub async fn start_seeded() -> (Self, String) {
        let bench = Self::start().await;
        let account = bench.add_account().await;
        let mirror = bench.open_mirror();
        seed_mirror(&mirror, &account);
        seed_bodies(&mirror);
        seed_threads(&mirror, &account);
        bench.deliver_server_mail();
        (bench, account)
    }

    /// Dépose sur le serveur les cinq courriers de la réception, sous les UID 1 à 5 comme le miroir.
    pub fn deliver_server_mail(&self) {
        let bodies = ["licorne rose", "rien", "rien", "", "encore une licorne"];
        for (index, body) in bodies.into_iter().enumerate() {
            let number = index as u32 + 1;
            let raw = if number == 4 {
                with_attachment()
            } else {
                plain(number, body)
            };
            self.store.deliver(USER, "INBOX", &raw, &[]);
        }
    }

    /// Ajoute le compte de test (mot de passe sur l'entrée standard) et rend son identifiant.
    pub async fn add_account(&self) -> String {
        let mut deps = self.deps(&format!("{PASSWORD}\n"));
        let arguments = [
            "account",
            "add",
            "--address",
            USER,
            "--password-stdin",
            "--json",
        ];
        let added = super::run_with(&mut deps, &arguments).await;
        assert_eq!(added.code, 0, "{}", added.stderr);
        super::account_id_of(&added)
    }

    /// Actions de la file du compte (`queue list`), tous états confondus.
    pub async fn queued_actions(&self, account: &str) -> Vec<Value> {
        let data = self.run_ok(&["queue", "list", account]).await;
        data["data"]["data"].as_array().unwrap().clone()
    }

    /// Ouvre la base du profil (le moteur ne doit pas tourner pendant ce temps).
    pub fn open_mirror(&self) -> Database {
        let deps = self.deps("");
        let config = deps.engine_config(ProfileId::from_raw("default"));
        let paths = ProfilePaths::resolve(&config.profile, &config.environment).unwrap();
        Database::open(&paths.database).unwrap()
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
