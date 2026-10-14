//! Modifications d'un brouillon : seuls les champs présents changent.

use mira_vocabulary::{AddressEntry, IdentityId};
use serde::{Deserialize, Serialize};

/// Champs à changer ; un `body_html` vide efface la version HTML.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftChanges {
    #[serde(default)]
    pub identity: Option<IdentityId>,
    #[serde(default)]
    pub to: Option<Vec<AddressEntry>>,
    #[serde(default)]
    pub cc: Option<Vec<AddressEntry>>,
    #[serde(default)]
    pub bcc: Option<Vec<AddressEntry>>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub body_text: Option<String>,
    #[serde(default)]
    pub body_html: Option<String>,
}
