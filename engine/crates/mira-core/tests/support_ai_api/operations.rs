//! Raccourcis d'opérations du domaine « Ai » sur une connexion déjà créée.

use mira_core::api::{AiRequest, AiResponse, Engine, SecretInput};
use mira_vocabulary::{AiConnection, AiConnectionId, AiModelList, ContentKind};

use super::calls::{call, call_connection};

/// Range cette clé pour la connexion, ou rend le code d'erreur
pub async fn set_key(
    engine: &Engine,
    connection: &AiConnectionId,
    key: &str,
) -> Result<AiConnection, String> {
    let request = AiRequest::SetAiApiKey {
        connection: connection.clone(),
        key: SecretInput::new(key),
    };
    call_connection(engine, request).await
}

/// Désigne la connexion par défaut
pub async fn set_default(engine: &Engine, connection: &AiConnectionId) -> AiConnection {
    let request = AiRequest::SetDefaultAiConnection {
        connection: connection.clone(),
    };
    call_connection(engine, request).await.unwrap()
}

/// Remplace libellé, adresse et modèle de la connexion, ou rend le code d'erreur
pub async fn update(
    engine: &Engine,
    connection: &AiConnectionId,
    label: &str,
    base_url: Option<&str>,
    default_model: Option<&str>,
) -> Result<AiConnection, String> {
    let request = AiRequest::UpdateAiConnection {
        connection: connection.clone(),
        label: label.to_string(),
        base_url: base_url.map(str::to_string),
        default_model: default_model.map(str::to_string),
    };
    call_connection(engine, request).await
}

/// Liste les modèles de la connexion, ou rend le code d'erreur
pub async fn models(
    engine: &Engine,
    connection: &AiConnectionId,
    refresh: bool,
) -> Result<AiModelList, String> {
    let request = AiRequest::ListAiModels {
        connection: connection.clone(),
        refresh,
    };
    match call(engine, request).await? {
        AiResponse::Models(list) => Ok(list),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Enregistre le consentement de l'utilisateur pour la connexion
pub async fn consent(engine: &Engine, connection: &AiConnectionId) -> AiConnection {
    let request = AiRequest::AcknowledgeAiConsent {
        connection: connection.clone(),
        kinds: vec![ContentKind::MailMessage],
    };
    call_connection(engine, request).await.unwrap()
}

/// Toutes les opérations qui n'ont besoin que d'une connexion existante, sauf la clé et la mise à jour
pub fn requests_on(connection: &AiConnectionId) -> Vec<AiRequest> {
    let connection = connection.clone();
    vec![
        AiRequest::RemoveAiConnection {
            connection: connection.clone(),
        },
        AiRequest::SetDefaultAiConnection {
            connection: connection.clone(),
        },
        AiRequest::ClearAiApiKey {
            connection: connection.clone(),
        },
        AiRequest::TestAiConnection {
            connection: connection.clone(),
        },
        AiRequest::AcknowledgeAiConsent {
            connection: connection.clone(),
            kinds: vec![ContentKind::MailMessage],
        },
        AiRequest::ListAiModels {
            connection,
            refresh: false,
        },
    ]
}
