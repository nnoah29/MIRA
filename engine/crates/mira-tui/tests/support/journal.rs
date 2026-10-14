//! Journal d'événements volumineux et ajout d'une boîte « pendant » le balayage du journal par le TUI.

use std::path::{Path, PathBuf};

use mira_assembly::mira_vocabulary::{
    AccountId, EventKind, Mailbox, MailboxId, MailboxRole, MiraError, OperationId, Timestamp,
};
use mira_assembly::profile::ProfilePaths;
use mira_assembly::storage::Database;

use super::world::TestWorld;

impl TestWorld {
    /// Chemin de la base du profil, pour écrire depuis une couture de test.
    pub fn database_path(&self) -> Result<PathBuf, MiraError> {
        let paths = ProfilePaths::resolve(&self.config.profile, &self.config.environment)?;
        Ok(paths.database)
    }

    /// Ajoute `count` événements anciens (fins de synchro) au journal, sans les diffuser.
    pub fn seed_old_events(&self, count: usize) -> Result<(), MiraError> {
        let account = self
            .account()
            .unwrap_or_else(|| AccountId::from_raw("acc-seed"));
        let kind = EventKind::SyncFinished {
            account,
            operation: OperationId::from_raw("op-ancienne"),
        };
        Database::open(&self.database_path()?)?.write(|tx| {
            for _ in 0..count {
                tx.append_event(Timestamp::from_millis(1_000), &kind)?;
            }
            Ok(())
        })
    }
}

/// Crée la boîte `name` (rôle « autre ») du compte et journalise sa création, comme une synchro le ferait.
pub fn add_mailbox_with_event(
    database: &Path,
    account: &AccountId,
    name: &str,
) -> Result<(), MiraError> {
    let mailbox = Mailbox {
        id: MailboxId::from_raw(format!("box-{name}")),
        account: account.clone(),
        name: name.to_string(),
        display_name: name.to_string(),
        delimiter: Some("/".to_string()),
        parent: None,
        role: MailboxRole::Other,
        sync_enabled: true,
        marker: None,
        total: 0,
        unread: 0,
    };
    let kind = EventKind::MailboxCreated {
        account: account.clone(),
        mailbox: mailbox.id.clone(),
    };
    Database::open(database)?.write(|tx| {
        tx.upsert_mailbox(&mailbox)?;
        tx.append_event(Timestamp::from_millis(2_000), &kind)?;
        Ok(())
    })
}
