//! Aides communes aux tests du dépôt des connexions d'IA.

pub mod profile;

use mira_core::storage::Database;
use mira_vocabulary::{AiConnection, AiConnectionId, AiProviderId, MaskingSettings, MiraError};

/// Connexion de test portant ce libellé, sans défaut ni consentement
pub fn connection(id: &str, label: &str) -> AiConnection {
    AiConnection {
        id: AiConnectionId::from_raw(id),
        provider: AiProviderId("test-provider".to_string()),
        label: label.to_string(),
        base_url: Some("https://ia.example.test/v1".to_string()),
        default_model: Some("modele-a".to_string()),
        is_default: false,
        has_key: false,
        consent_at: None,
        models: None,
        allowed_kinds: Vec::new(),
        masking: MaskingSettings::remote_default(),
    }
}

/// Insère la connexion dans une base déjà ouverte
pub fn insert(db: &Database, item: &AiConnection) -> Result<(), MiraError> {
    db.write(|tx| tx.insert_ai_connection(item, 1))
}

/// Code de l'erreur rendue
pub fn code_of(result: Result<(), MiraError>) -> String {
    result.unwrap_err().code
}

/// Identifiants des connexions listées, dans l'ordre rendu
pub fn listed_ids(db: &Database) -> Vec<String> {
    db.read(|tx| tx.list_ai_connections())
        .unwrap()
        .into_iter()
        .map(|item| item.id.as_str().to_string())
        .collect()
}
