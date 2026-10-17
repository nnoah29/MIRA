//! Rendu humain des flux OAuth et des opérations longues.

use mira_assembly::api::contract::OAuthStart;
use mira_assembly::operations::OperationInfo;

use super::label::json_label;

/// Identifiant du flux et adresse d'autorisation à ouvrir dans un navigateur.
pub fn oauth_start_text(start: &OAuthStart) -> String {
    format!(
        "Flux : {}\nAutorisation : {}",
        start.flow, start.authorize_url
    )
}

/// Identifiant, nature et avancement d'une opération.
pub fn operation_text(info: &OperationInfo) -> String {
    format!(
        "Opération : {}\nNature : {}\nÉtat : {}",
        info.id,
        info.kind,
        json_label(&info.status)
    )
}
