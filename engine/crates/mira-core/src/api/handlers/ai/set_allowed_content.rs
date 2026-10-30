//! Types de contenu qu'une connexion d'IA peut recevoir : la liste donnée remplace l'ancienne.

use mira_vocabulary::{AiConnectionId, ContentKind, MiraError};

use super::allowed_kinds::distinct_kinds;
use super::connection_view::{read_connection, single};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Remplace la liste des types autorisés, sans doublon ; inconnue : `not_found.ai_connection`
pub(crate) fn handle_set_ai_allowed_content(
    engine: &Engine,
    id: &AiConnectionId,
    kinds: &[ContentKind],
) -> Result<Response, MiraError> {
    let kinds = distinct_kinds(kinds);
    engine
        .database
        .write(|tx| tx.set_ai_allowed_kinds(id, &kinds))?;
    Ok(single(read_connection(engine, id)?))
}
