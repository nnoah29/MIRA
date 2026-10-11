//! Client IMAP de haut niveau : état de la session et aides communes aux opérations.

use mira_vocabulary::error::codes::server;
use mira_vocabulary::{ErrorCategory, MiraError};

use super::errors::status::Operation;
use super::protocol::response::Response;
use super::protocol::wire::Wire;
use crate::imap::errors::status::status_error;
use crate::imap::protocol::response::StatusKind;

/// Session IMAP authentifiée ; les opérations travaillent sur la boîte sélectionnée.
#[derive(Debug)]
pub struct ImapClient {
    pub(crate) wire: Wire,
    pub(crate) capabilities: Vec<String>,
    pub(crate) selected: Option<String>,
    pub(crate) qresync: bool,
}

impl ImapClient {
    /// Assemble un client à partir d'un flux authentifié et de ses capacités.
    pub(crate) fn from_parts(mut wire: Wire, capabilities: Vec<String>) -> Self {
        wire.set_literal_plus(capabilities.iter().any(|name| name == "LITERAL+"));
        Self {
            wire,
            capabilities,
            selected: None,
            qresync: false,
        }
    }

    /// Capacités annoncées par le serveur après authentification (en majuscules).
    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    /// Indique si le serveur a annoncé la capacité.
    pub(crate) fn has_capability(&self, name: &str) -> bool {
        self.capabilities.iter().any(|known| known == name)
    }

    /// Exige une capacité ; sinon l'extension est signalée absente sans envoyer de commande.
    pub(crate) fn require_capability(&self, name: &str) -> Result<(), MiraError> {
        if self.has_capability(name) {
            return Ok(());
        }
        Err(MiraError::new(
            ErrorCategory::Server,
            server::EXTENSION_MISSING,
            format!("le serveur ne prend pas en charge {name}"),
        ))
    }

    /// Exige qu'une boîte soit sélectionnée avant l'opération.
    pub(crate) fn require_selected(&self) -> Result<(), MiraError> {
        if self.selected.is_some() {
            return Ok(());
        }
        Err(MiraError::new(
            ErrorCategory::Internal,
            mira_vocabulary::error::codes::internal::BUG,
            "aucune boîte sélectionnée",
        ))
    }

    /// Envoie une commande et rend sa réponse si elle est `OK`.
    pub(crate) async fn run(
        &mut self,
        line: &str,
        operation: Operation,
    ) -> Result<Response, MiraError> {
        let response = self.wire.command(line).await?;
        if response.status.kind == StatusKind::Ok {
            return Ok(response);
        }
        Err(status_error(&response.status, operation))
    }
}
