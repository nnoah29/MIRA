//! Exemples du domaine « réglages des libellés » : liste, réglages partiels, ordre.

use mira_core::api::{
    LabelOverview, LabelSettingsRequest, LabelSettingsResponse, Request, Response,
};
use mira_vocabulary::{AccountId, CounterMode, LabelKind, LabelRef, LabelSettings};

use super::{Named, numbered};

/// Libellé d'exemple
fn sample_ref() -> LabelRef {
    LabelRef {
        kind: LabelKind::Server,
        id: "mailbox-1".to_string(),
    }
}

/// Une requête par opération de `LabelSettingsRequest`
pub fn requests() -> Vec<Named<Request>> {
    let items = vec![
        LabelSettingsRequest::ListLabels,
        LabelSettingsRequest::SetLabelSettings {
            label: sample_ref(),
            hidden: Some(true),
            hide_from_inbox: Some(true),
            notifications: Some(false),
            counter: Some(CounterMode::Total),
            color: Some(0xFF6A_A8D9),
        },
        LabelSettingsRequest::ReorderLabels {
            order: vec![sample_ref()],
        },
    ];
    let ops: Vec<&str> = items.iter().map(op_name).collect();
    numbered("label_settings", &ops)
        .into_iter()
        .zip(items.into_iter().map(Request::LabelSettings))
        .collect()
}

/// Une réponse par forme de `LabelSettingsResponse`
pub fn responses() -> Vec<Named<Response>> {
    let overview = LabelOverview {
        label: sample_ref(),
        account: Some(AccountId::from_raw("account-1")),
        name: "Factures".to_string(),
        color: 0xFFE8_542B,
        messages: 12,
        unused: false,
        settings: LabelSettings::default(),
    };
    let items = [
        (
            "label_settings.labels",
            LabelSettingsResponse::Labels(vec![overview.clone()]),
        ),
        (
            "label_settings.label",
            LabelSettingsResponse::Label(overview),
        ),
        ("label_settings.done", LabelSettingsResponse::Done),
    ];
    let wrap = |(name, inner): (&str, LabelSettingsResponse)| {
        (name.to_string(), Response::LabelSettings(inner))
    };
    items.into_iter().map(wrap).collect()
}

/// Nom de l'opération, par `match` exhaustif
pub fn op_name(request: &LabelSettingsRequest) -> &'static str {
    match request {
        LabelSettingsRequest::ListLabels => "list_labels",
        LabelSettingsRequest::SetLabelSettings { .. } => "set_label_settings",
        LabelSettingsRequest::ReorderLabels { .. } => "reorder_labels",
    }
}
