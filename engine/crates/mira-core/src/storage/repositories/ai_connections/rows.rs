//! Lignes de `ai_connections` et colonnes de la liste de modèles gardée.

use mira_vocabulary::{
    AiConnection, AiConnectionId, AiModelList, AiProviderId, MiraError, Timestamp,
};
use rusqlite::Row;

use crate::storage::json_columns::{from_json, to_json};

pub(super) const COLUMNS: &str = "id, provider_id, label, base_url, default_model, is_default, consent_at, models_json, models_fetched_at";

/// Lit une ligne dans l'ordre de `COLUMNS` ; la liste de modèles illisible est une erreur de stockage
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
    };
    let models = models_from_columns(row.get(7)?, row.get(8)?);
    Ok(models.map(|models| AiConnection {
        models,
        ..connection
    }))
}

/// Reconstitue la liste de modèles à partir de ses deux colonnes
pub(super) fn models_from_columns(
    json: Option<String>,
    fetched_at: Option<i64>,
) -> Result<Option<AiModelList>, MiraError> {
    match (json, fetched_at) {
        (Some(text), Some(millis)) => Ok(Some(AiModelList {
            models: from_json(&text)?,
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
