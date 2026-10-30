//! Exécution des requêtes d'un outil par la façade du moteur, réservée aux lectures.

use mira_vocabulary::{ErrorCode, MiraError, ToolRisk};

use crate::api::contract::risk_of;
use crate::api::{Engine, Request, Response};

/// Pourquoi une lecture n'a rien donné : requête refusée par la garde, ou erreur du moteur.
#[derive(Debug)]
pub(super) enum ReadFailure {
    NotRead,
    Engine(MiraError),
}

impl ReadFailure {
    /// Explication donnée au modèle : jamais de détail interne, seulement le code stable.
    pub(super) fn text(&self) -> String {
        match self {
            Self::NotRead => "opération non autorisée pour un outil".to_string(),
            Self::Engine(error) => format!("erreur du moteur : {}", error.code),
        }
    }

    /// Dit si l'erreur du moteur porte ce code.
    pub(super) fn is_code(&self, code: &ErrorCode) -> bool {
        matches!(self, Self::Engine(error) if error.code == code.0)
    }
}

/// Dit si la requête est une lecture seule, seule sorte exposable aux outils.
pub fn is_read(request: &Request) -> bool {
    risk_of(request) == ToolRisk::Read
}

/// Exécute une lecture ; une requête qui n'en est pas une est refusée avant tout accès.
pub(super) async fn execute_read(
    engine: &Engine,
    request: Request,
) -> Result<Response, ReadFailure> {
    if !is_read(&request) {
        return Err(ReadFailure::NotRead);
    }
    engine.execute(request).await.map_err(ReadFailure::Engine)
}
