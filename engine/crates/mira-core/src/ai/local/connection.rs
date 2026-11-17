//! Connexion du modèle local : créée avec le premier modèle installé, retirée avec le dernier.

use mira_vocabulary::{
    AiConnection, AiConnectionId, AiProviderDescriptor, MaskingSettings, MiraError,
};

use crate::ports::{Clock, IdGenerator};
use crate::storage::Database;

/// Crée la connexion du fournisseur local si elle manque, et y met ce modèle par défaut.
pub(crate) fn ensure_local_connection(
    database: &Database,
    ids: &dyn IdGenerator,
    clock: &dyn Clock,
    descriptor: &AiProviderDescriptor,
    model_id: &str,
) -> Result<(), MiraError> {
    let now = clock.now().as_millis();
    let existing = find_connection(database, descriptor)?;
    database.write(|tx| match &existing {
        Some(found) => tx.update_ai_connection(
            &found.id,
            &found.label,
            found.base_url.as_deref(),
            Some(model_id),
            now,
        ),
        None => tx.insert_ai_connection(&new_connection(ids, descriptor, model_id), now),
    })
}

/// Connexion du fournisseur local, si elle existe.
fn find_connection(
    database: &Database,
    descriptor: &AiProviderDescriptor,
) -> Result<Option<AiConnection>, MiraError> {
    let all = database.read(|tx| tx.list_ai_connections())?;
    Ok(all.into_iter().find(|c| c.provider == descriptor.id))
}

/// Connexion neuve : ni défaut, ni consentement (rien ne sort de la machine), masquage local.
fn new_connection(
    ids: &dyn IdGenerator,
    descriptor: &AiProviderDescriptor,
    model_id: &str,
) -> AiConnection {
    AiConnection {
        id: AiConnectionId::from_raw(ids.next_id()),
        provider: descriptor.id.clone(),
        label: descriptor.display_name.clone(),
        base_url: None,
        default_model: Some(model_id.to_string()),
        is_default: false,
        has_key: false,
        consent_at: None,
        models: None,
        allowed_kinds: Vec::new(),
        masking: MaskingSettings::local_default(),
    }
}

/// Après un retrait : supprime la connexion s'il ne reste aucun modèle, sinon corrige son modèle par défaut.
pub(crate) fn trim_local_connection(
    database: &Database,
    clock: &dyn Clock,
    descriptor: &AiProviderDescriptor,
    removed: &str,
    remaining: &[String],
) -> Result<(), MiraError> {
    let Some(found) = find_connection(database, descriptor)? else {
        return Ok(());
    };
    let now = clock.now().as_millis();
    database.write(|tx| match remaining.first() {
        None => tx.delete_ai_connection(&found.id),
        Some(other) if found.default_model.as_deref() == Some(removed) => tx.update_ai_connection(
            &found.id,
            &found.label,
            found.base_url.as_deref(),
            Some(other),
            now,
        ),
        Some(_) => Ok(()),
    })
}
