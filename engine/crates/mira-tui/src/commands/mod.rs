//! Effets demandés par `update` et exécutés par la boucle d'événements.

use mira_assembly::api::Request;

/// Étiquette qui relie une réponse du moteur à l'intention qui l'a demandée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Startup,
}

/// Effet de bord que la boucle doit réaliser après une transition.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    Execute { tag: Tag, request: Request },
    Quit,
}
