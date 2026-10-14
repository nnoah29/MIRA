//! Commandes à exécuter au démarrage de l'application.

use mira_assembly::api::{AccountRequest, Request};

use super::AppState;
use crate::commands::{Command, Tag};

impl AppState {
    /// Commandes de démarrage : charger la liste des comptes.
    pub fn boot_commands(&self) -> Vec<Command> {
        vec![Command::Execute {
            tag: Tag::Accounts,
            request: Request::Account(AccountRequest::ListAccounts),
        }]
    }
}
