//! Exemples du domaine « réglages du moteur » : lecture et remplacement.

use mira_core::api::{Request, Response, SettingsRequest, SettingsResponse};
use mira_vocabulary::{AccountId, EngineSettings, QuietHours};

use super::{Named, numbered};

/// Réglages d'exemple
fn sample() -> EngineSettings {
    EngineSettings {
        poll_secs: Some(120),
        muted_accounts: vec![AccountId::from_raw("account-1")],
        quiet_hours: Some(QuietHours { from: 22, to: 7 }),
    }
}

/// Une requête par opération de `SettingsRequest`
pub fn requests() -> Vec<Named<Request>> {
    let items = vec![
        SettingsRequest::GetSettings,
        SettingsRequest::SetSettings { settings: sample() },
    ];
    let ops: Vec<&str> = items.iter().map(op_name).collect();
    numbered("settings", &ops)
        .into_iter()
        .zip(items.into_iter().map(Request::Settings))
        .collect()
}

/// Une réponse par forme de `SettingsResponse`
pub fn responses() -> Vec<Named<Response>> {
    let inner = SettingsResponse::Settings(sample());
    vec![("settings.settings".to_string(), Response::Settings(inner))]
}

/// Nom de l'opération, par `match` exhaustif
pub fn op_name(request: &SettingsRequest) -> &'static str {
    match request {
        SettingsRequest::GetSettings => "get_settings",
        SettingsRequest::SetSettings { .. } => "set_settings",
    }
}
