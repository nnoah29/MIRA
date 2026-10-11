//! État d'une session : utilisateur, boîte sélectionnée, extensions activées.

use std::sync::Arc;

use crate::imap::capabilities::Capability;
use crate::imap::response::error::ImapError;
use crate::imap::server::context::ServerContext;

/// Extensions négociées par le client avec ENABLE ou implicitement.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Enabled {
    pub condstore: bool,
    pub qresync: bool,
}

/// Boîte sélectionnée et ce que le client en sait.
#[derive(Debug, Clone)]
pub(crate) struct Selected {
    pub name: String,
    pub read_only: bool,
    pub uids: Vec<u32>,
    pub synced_modseq: u64,
    pub validity: u32,
}

/// État complet d'une session.
#[derive(Debug)]
pub(crate) struct SessionState {
    pub ctx: Arc<ServerContext>,
    pub user: Option<String>,
    pub selected: Option<Selected>,
    pub enabled: Enabled,
}

impl SessionState {
    /// État initial d'une connexion non authentifiée
    pub(crate) fn new(ctx: Arc<ServerContext>) -> Self {
        Self {
            ctx,
            user: None,
            selected: None,
            enabled: Enabled::default(),
        }
    }

    /// Utilisateur authentifié, sinon BAD
    pub(crate) fn require_user(&self) -> Result<String, ImapError> {
        self.user
            .clone()
            .ok_or_else(|| ImapError::bad("Authenticate first"))
    }

    /// Boîte sélectionnée, sinon BAD
    pub(crate) fn require_selected(&self) -> Result<&Selected, ImapError> {
        self.selected
            .as_ref()
            .ok_or_else(|| ImapError::bad("No mailbox selected"))
    }

    /// Extension annoncée par le serveur, sinon BAD
    pub(crate) fn require_cap(&self, capability: Capability) -> Result<(), ImapError> {
        if self.ctx.caps.contains(capability) {
            Ok(())
        } else {
            Err(ImapError::bad("Command not recognized"))
        }
    }

    /// Utilisateur et nom de la boîte sélectionnée
    pub(crate) fn target(&self) -> Result<(String, String), ImapError> {
        let selected = self.require_selected()?;
        Ok((self.require_user()?, selected.name.clone()))
    }
}
