//! Réponses du domaine « réglages des libellés » : vue d'ensemble d'un libellé et liste.

use mira_vocabulary::{AccountId, LabelRef, LabelSettings};
use serde::{Deserialize, Serialize};

/// Un libellé tel que l'onglet des libellés l'affiche : identité, couleur effective, volume et réglages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelOverview {
    pub label: LabelRef,
    pub account: Option<AccountId>,
    pub name: String,
    pub color: u32,
    pub messages: u64,
    pub unused: bool,
    pub settings: LabelSettings,
}

/// Réponse à une requête du domaine réglages des libellés.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum LabelSettingsResponse {
    Labels(Vec<LabelOverview>),
    Label(LabelOverview),
    Done,
}
