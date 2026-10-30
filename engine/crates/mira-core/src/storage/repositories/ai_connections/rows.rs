//! Lignes de `ai_connections` et colonnes de la liste de modèles gardée.

use mira_vocabulary::{
    AiConnection, AiConnectionId, AiModelList, AiProviderId, ContentKind, MaskingSettings,
    MiraError, Timestamp,
};
use rusqlite::Row;

use crate::storage::json_columns::{from_json, to_json};

pub(super) const COLUMNS: &str = "id, provider_id, label, base_url, default_model, is_default, consent_at, models_json, models_fetched_at, allowed_kinds, masking_json";

/// Lit une ligne dans l'ordre de `COLUMNS` ; une liste de modèles illisible donne `models: None`
pub(super) fn connection_from_row(
    row: &Row<'_>,
) -> rusqlite::Result<Result<AiConnection, MiraError>> {
    let consent_at: Option<i64> = row.get(6)?;
    let connection = AiConnection {
        id: AiConnectionId::from_raw(row.get::<_, String>(0)?),
        provider: AiProviderId(row.get(1)?),
        label: row.get(2)?,
        base_url: row.get(3)?,
        default_model: row.get(4)?,
        is_default: row.get::<_, i64>(5)? != 0,
        has_key: false,
        consent_at: consent_at.map(Timestamp::from_millis),
        models: None,
        allowed_kinds: Vec::new(),
        masking: MaskingSettings::default(),
    };
    let (allowed_kinds, masking) =
        privacy_from_columns(&row.get::<_, String>(9)?, &row.get::<_, String>(10)?);
    let connection = AiConnection {
        allowed_kinds,
        masking,
        ..connection
    };
    let models = models_from_columns(row.get(7)?, row.get(8)?);
    Ok(models.map(|models| AiConnection {
        models,
        ..connection
    }))
}

/// Reconstitue la liste de modèles à partir de ses deux colonnes ; un cache illisible est jeté, il se relève
pub(super) fn models_from_columns(
    json: Option<String>,
    fetched_at: Option<i64>,
) -> Result<Option<AiModelList>, MiraError> {
    match (json, fetched_at) {
        (Some(text), Some(millis)) => Ok(from_json(&text).ok().map(|models| AiModelList {
            models,
            fetched_at: Timestamp::from_millis(millis),
        })),
        _ => Ok(None),
    }
}

/// Colonnes JSON et date de relevé d'une liste de modèles éventuelle
pub(super) fn models_to_columns(
    list: Option<&AiModelList>,
) -> Result<(Option<String>, Option<i64>), MiraError> {
    match list {
        Some(found) => Ok((
            Some(to_json(&found.models)?),
            Some(found.fetched_at.as_millis()),
        )),
        None => Ok((None, None)),
    }
}

/// Relit types autorisés et masquage ; une colonne illisible donne le réglage le plus protecteur
pub(super) fn privacy_from_columns(
    kinds: &str,
    masking: &str,
) -> (Vec<ContentKind>, MaskingSettings) {
    (
        from_json(kinds).unwrap_or_default(),
        from_json(masking).unwrap_or_default(),
    )
}

/// Colonnes JSON des types autorisés et du masquage d'une connexion
pub(super) fn privacy_to_columns(connection: &AiConnection) -> Result<(String, String), MiraError> {
    Ok((
        to_json(&connection.allowed_kinds)?,
        to_json(&connection.masking)?,
    ))
}
