//! Gestes de synchro des scénarios : lancer la passe, ajouter un compte qui l'attend, lire les repères.

use serde_json::Value;

use super::sa_world::SaWorld;
use super::{account_id_of, parse_json, run_with};

impl SaWorld {
    /// Lance `sync start` (attendu) et rend le champ `data` de la réponse.
    pub async fn first_pass(&self) -> Value {
        self.run_sync_ok(&["sync", "start"]).await
    }

    /// Ajoute le compte avec `--sync-wait` : l'ajout attend la première synchro. Rend l'identifiant.
    pub async fn add_account_waiting_first_sync(&self) -> String {
        let mut deps = self.sync_deps(&format!("{}\n", self.password));
        let line = [
            "account",
            "add",
            "--address",
            &self.user,
            "--password-stdin",
        ];
        let line = [&line[..], &["--sync-wait", "--json"]].concat();
        let added = run_with(&mut deps, &line).await;
        assert_eq!(added.code, 0, "{}", added.stderr);
        self.transcript.lock().unwrap().push(added.stdout.clone());
        assert_eq!(parse_json(&added)["ok"], true);
        account_id_of(&added)
    }

    /// Fiche de la boîte de ce nom (compteurs, repère), telle que `mailbox list` la rend.
    pub async fn mailbox_view(&self, account: &str, name: &str) -> Value {
        let listed = self.run_ok(&["mailbox", "list", account]).await;
        let boxes = listed["data"].as_array().unwrap();
        let found = boxes.iter().find(|item| item["name"] == name).unwrap();
        found.clone()
    }
}
