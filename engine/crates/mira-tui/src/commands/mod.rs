//! Effets demandés par `update` et exécutés par la boucle d'événements.

use mira_assembly::api::Request;
use mira_assembly::mira_vocabulary::{AccountId, ViewRef};

/// Étiquette qui relie une réponse du moteur à l'intention qui l'a demandée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tag {
    Startup,
    Accounts,
    Mailboxes(AccountId),
    Messages { view: ViewRef, append: bool },
}

/// Effet de bord que la boucle doit réaliser après une transition.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    Execute { tag: Tag, request: Request },
    Quit,
}
