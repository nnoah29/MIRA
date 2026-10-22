//! Requêtes du domaine « réglages des libellés » (spec R3 §4) : liste, réglages partiels, ordre.

use mira_vocabulary::{CounterMode, LabelRef};
use serde::{Deserialize, Serialize};

/// Opérations sur les réglages des libellés ; seuls les champs fournis sont modifiés.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum LabelSettingsRequest {
    ListLabels,
    SetLabelSettings {
        label: LabelRef,
        hidden: Option<bool>,
        hide_from_inbox: Option<bool>,
        notifications: Option<bool>,
        counter: Option<CounterMode>,
        color: Option<u32>,
    },
    ReorderLabels {
        order: Vec<LabelRef>,
    },
}
