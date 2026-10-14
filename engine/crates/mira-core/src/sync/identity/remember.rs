//! Mémorisation des identifiants serveur d'un message reconnu ou créé (table de correspondance D-13).

use mira_vocabulary::{AccountId, MessageId, MiraError};

use super::incoming::Incoming;
use crate::storage::Tx;
use crate::storage::repositories::server_ids::ServerIdKind;

/// Associe au message son identifiant stable (s'il existe) et sa clé « boîte/validité/uid ».
pub fn remember(
    tx: &Tx<'_>,
    account: &AccountId,
    message: &MessageId,
    incoming: &Incoming,
) -> Result<(), MiraError> {
    tx.map_server_id(account, ServerIdKind::Uid, &incoming.uid_key(), message)?;
    if let Some(stable) = &incoming.translation.stable_id {
        tx.map_server_id(account, ServerIdKind::Stable, stable, message)?;
    }
    Ok(())
}
