//! Rappel des touches de chaque page de la vue Paramètres, pour le pied de page.

use super::hints::Hint;
use crate::state::settings::SettingsTab;

const ACCOUNTS: &[Hint] = &[
    Hint { keys: "E", action: "modifier" },
    Hint { keys: "p/r", action: "pause/reprise" },
    Hint { keys: "X", action: "supprimer" },
    Hint { keys: "Tab", action: "page suivante" },
    Hint { keys: "Échap", action: "fermer" },
];

const IDENTITIES: &[Hint] = &[
    Hint { keys: "a", action: "ajouter" },
    Hint { keys: "e", action: "modifier" },
    Hint { keys: "d", action: "par défaut" },
    Hint { keys: "]/[", action: "compte" },
    Hint { keys: "Échap", action: "fermer" },
];

const MAINTENANCE: &[Hint] = &[
    Hint { keys: "s", action: "volumes" },
    Hint { keys: "c", action: "intégrité" },
    Hint { keys: "p", action: "vider le cache" },
    Hint { keys: "i", action: "reconstruire l'index" },
    Hint { keys: "d/x", action: "exporter diagnostic/profil" },
];

const EVENTS: &[Hint] = &[
    Hint { keys: "n/p", action: "page suivante/précédente" },
    Hint { keys: "Home/G", action: "début/fin du journal" },
    Hint { keys: "j/k", action: "défiler" },
    Hint { keys: "Échap", action: "fermer" },
];

/// Touches propres à cette page, les plus utiles d'abord.
pub fn settings_hints(tab: SettingsTab) -> &'static [Hint] {
    match tab {
        SettingsTab::Accounts => ACCOUNTS,
        SettingsTab::Identities => IDENTITIES,
        SettingsTab::Maintenance => MAINTENANCE,
        SettingsTab::Events => EVENTS,
    }
}
