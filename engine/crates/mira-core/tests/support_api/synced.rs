//! Moteur déjà synchronisé une fois, et redémarrage du moteur sur le même profil.

use mira_core::api::Engine;
use mira_core::storage::Database;
use mira_vocabulary::{AccountId, CONTRACT_VERSION};

use super::fixture::{FOLDERS_USER, Fixture};
use super::sync_calls::{finished_passes, wait_for_events};
use super::syncing::{days_ago, deliver_many, mail, start_syncing};

/// Attend la première passe terminée et les deux connexions du compte (travail et IDLE), au plus 15 s
async fn wait_for_listening(f: &Fixture) {
    wait_for_events(&f.engine, |k| finished_passes(k) >= 1).await;
    let wait = async {
        while f.imap.active_connections() != 2 {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    };
    let ready = tokio::time::timeout(std::time::Duration::from_secs(15), wait).await;
    ready.expect("l'écoute IDLE n'a jamais démarré");
}

/// Compte `folders` ajouté et déjà synchronisé une fois, avec une boîte « Archive » de deux messages
pub async fn synced_account() -> (Fixture, AccountId) {
    let f = start_syncing().await;
    f.store.create_mailbox(FOLDERS_USER, "Archive", None);
    deliver_many(&f, "Archive", "Ancien", 2);
    let account = f.add_folders_account("k-synced").await;
    wait_for_listening(&f).await;
    (f, account)
}

/// Compte `folders` synchronisé une fois : deux messages récents en réception, quatre plus anciens restés sur le serveur
pub async fn aged_account() -> (Fixture, AccountId) {
    let f = start_syncing().await;
    let ages = [(1, "Récent A"), (2, "Récent B"), (100, "Ancien A")];
    let older = [(100, "Ancien B"), (101, "Ancien C"), (200, "Très ancien")];
    for (index, (age, subject)) in ages.into_iter().chain(older).enumerate() {
        let raw = mail(subject, &format!("age{index}@x"));
        let user = FOLDERS_USER;
        f.store
            .deliver_dated(user, "INBOX", &raw, &[], days_ago(age));
    }
    let account = f.add_folders_account("k-aged").await;
    wait_for_listening(&f).await;
    (f, account)
}

impl Fixture {
    /// Ferme le moteur puis en ouvre un autre sur le même profil, comme après un redémarrage
    pub fn reopen(self) -> Fixture {
        self.engine.shutdown().unwrap();
        let engine = Engine::open(self.config.clone(), CONTRACT_VERSION).unwrap();
        Fixture { engine, ..self }
    }

    /// Ferme le moteur, laisse le compte « en suppression » comme après un arrêt brutal, puis rouvre le moteur
    pub fn reopen_after_crash_in_deletion(self, account: &AccountId) -> Fixture {
        self.engine.shutdown().unwrap();
        let path = self.home.path().join("profiles/default/mira.sqlite3");
        let database = Database::open(&path).unwrap();
        database
            .write(|tx| tx.mark_account_deleting(account))
            .unwrap();
        drop(database);
        let engine = Engine::open(self.config.clone(), CONTRACT_VERSION).unwrap();
        Fixture { engine, ..self }
    }
}
