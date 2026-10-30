//! Exécution des requêtes d'un outil par la façade du moteur, réservée aux lectures.

use mira_vocabulary::ToolRisk;

use crate::api::contract::risk_of;
use crate::api::{Engine, Request, Response};

/// Dit si la requête est une lecture seule, seule sorte exposable aux outils.
pub fn is_read(request: &Request) -> bool {
    risk_of(request) == ToolRisk::Read
}

/// Exécute une lecture ; une écriture est refusée, une erreur du moteur n'est rendue que par son code.
pub(super) async fn execute_read(engine: &Engine, request: Request) -> Result<Response, String> {
    if !is_read(&request) {
        return Err("opération non autorisée pour un outil".to_string());
    }
    engine
        .execute(request)
        .await
        .map_err(|error| format!("erreur du moteur : {}", error.code))
}
