//! Appels du domaine « label_settings » par la façade et lecture de leurs réponses.

use mira_core::api::{
    Engine, LabelOverview, LabelSettingsRequest, LabelSettingsResponse, Request, Response,
};
use mira_vocabulary::{CounterMode, EventKind, LabelKind, LabelRef};

use crate::support_api::queries::event_kinds;

/// Exécute une requête du domaine et rend la réponse du domaine, ou le code d'erreur
pub async fn call(
    engine: &Engine,
    request: LabelSettingsRequest,
) -> Result<LabelSettingsResponse, String> {
    match engine.execute(Request::LabelSettings(request)).await {
        Ok(Response::LabelSettings(answer)) => Ok(answer),
        Ok(other) => panic!("réponse inattendue : {other:?}"),
        Err(error) => Err(error.code),
    }
}

/// Référence d'un libellé du serveur
pub fn server_ref(id: &str) -> LabelRef {
    LabelRef {
        kind: LabelKind::Server,
        id: id.to_string(),
    }
}

/// Référence d'un libellé local
pub fn local_ref(id: &str) -> LabelRef {
    LabelRef {
        kind: LabelKind::Local,
        id: id.to_string(),
    }
}

/// Tous les libellés, dans l'ordre rendu par le moteur
pub async fn labels(engine: &Engine) -> Vec<LabelOverview> {
    match call(engine, LabelSettingsRequest::ListLabels)
        .await
        .unwrap()
    {
        LabelSettingsResponse::Labels(found) => found,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Réglage d'un seul champ booléen ou de compteur, les autres inchangés
pub async fn set(
    engine: &Engine,
    label: &LabelRef,
    hidden: Option<bool>,
    hide_from_inbox: Option<bool>,
    counter: Option<CounterMode>,
) -> Result<LabelOverview, String> {
    let request = LabelSettingsRequest::SetLabelSettings {
        label: label.clone(),
        hidden,
        hide_from_inbox,
        notifications: None,
        counter,
        color: None,
    };
    match call(engine, request).await? {
        LabelSettingsResponse::Label(shown) => Ok(shown),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Nombre d'événements « réglages des libellés changés »
pub async fn changed_count(engine: &Engine) -> usize {
    let kinds = event_kinds(engine).await;
    kinds
        .iter()
        .filter(|kind| matches!(kind, EventKind::LabelSettingsChanged))
        .count()
}
