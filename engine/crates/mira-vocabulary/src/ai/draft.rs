//! Modes de génération de l'assistant et propositions de brouillon que l'application applique seule.

use serde::{Deserialize, Serialize};

use crate::address::entry::AddressEntry;

/// Contexte d'une génération : le client choisit un jeu d'outils défini par le moteur, jamais un outil.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChatMode {
    #[default]
    Assistant,
    Composer {
        selection: bool,
    },
}

/// Ce que la proposition vise : un nouveau message, ou le brouillon ouvert dans le composeur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftProposalKind {
    New,
    Edit,
}

/// Brouillon proposé par le modèle, validé par le moteur ; seuls les champs présents changent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftProposal {
    pub kind: DraftProposalKind,
    pub to: Option<Vec<AddressEntry>>,
    pub cc: Option<Vec<AddressEntry>>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub selection: Option<String>,
    pub reply_to: Option<String>,
    pub identity: Option<String>,
}

impl DraftProposal {
    /// Proposition de ce genre sans aucun champ renseigné.
    pub fn empty(kind: DraftProposalKind) -> Self {
        Self {
            kind,
            to: None,
            cc: None,
            subject: None,
            body: None,
            selection: None,
            reply_to: None,
            identity: None,
        }
    }
}
