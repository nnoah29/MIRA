//! Outils d'IA : niveau de risque, appel demandé par le modèle, résultat renvoyé et état d'une exécution.

use serde::{Deserialize, Serialize};

/// Niveau de risque d'une opération du contrat quand un modèle peut la demander.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolRisk {
    Read,
    Reversible,
    Confirmed,
    Sensitive,
}

/// Appel d'outil demandé par le modèle ; `arguments` est du JSON en texte, validé avant exécution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// Résultat d'un appel, renvoyé au modèle au tour suivant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolOutcome {
    pub call_id: String,
    pub text: String,
    pub is_error: bool,
}

/// Issue d'une exécution d'outil, telle qu'annoncée au lecteur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolState {
    Ok,
    Denied,
    Error,
}
