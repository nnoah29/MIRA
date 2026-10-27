//! Réponses du domaine « règles » : règle avec son statut, aperçu, catalogue des conditions et actions.

use mira_vocabulary::{MessageId, Rule, RuleImportReport};
use serde::{Deserialize, Serialize};

use crate::rules::validate::RuleStatus;

/// Règle lue avec son statut calculé à la lecture (valide, ou invalide avec la raison).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleView {
    pub rule: Rule,
    pub status: RuleStatus,
}

/// Aperçu d'une règle sur les messages existants : touchés parmi les examinés, et quelques exemples.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RulePreview {
    pub matched: u64,
    pub scanned: u64,
    pub samples: Vec<MessageId>,
}

/// Une variante de condition ou d'action : étiquette JSON, famille, paramètres et besoins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub kind: String,
    pub family: String,
    pub fields: Vec<String>,
    pub needs_mailbox: bool,
    pub single_account: bool,
}

/// Catalogue complet des conditions et des actions connues du moteur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleCatalog {
    pub conditions: Vec<CatalogEntry>,
    pub actions: Vec<CatalogEntry>,
}

/// Réponse à une requête du domaine règles ; une suppression répond `Done`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum RuleResponse {
    Rules(Vec<RuleView>),
    Rule(RuleView),
    Status(RuleStatus),
    Preview(RulePreview),
    Catalog(RuleCatalog),
    RulesExport { yaml: String, skipped: Vec<String> },
    RulesImport(RuleImportReport),
}
