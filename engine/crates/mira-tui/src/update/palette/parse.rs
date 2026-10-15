//! Analyse d'une commande de la palette : un mot connu, rien d'autre.

use crate::state::settings::SettingsTab;
use crate::update::settings::SettingsIntent;
use crate::update::{DraftsIntent, Intent, QueueIntent};

/// Commandes connues de la palette et l'intention qu'elles déclenchent.
pub const COMMANDS: [(&str, Intent); 8] = [
    ("queue", Intent::Queue(QueueIntent::Open)),
    ("drafts", Intent::Drafts(DraftsIntent::Open)),
    ("sync", Intent::StartSync),
    ("help", Intent::Help),
    (
        "settings",
        Intent::Settings(SettingsIntent::Show(SettingsTab::Accounts)),
    ),
    (
        "identities",
        Intent::Settings(SettingsIntent::Show(SettingsTab::Identities)),
    ),
    (
        "maintenance",
        Intent::Settings(SettingsIntent::Show(SettingsTab::Maintenance)),
    ),
    (
        "events",
        Intent::Settings(SettingsIntent::Show(SettingsTab::Events)),
    ),
];

/// Intention d'une commande tapée, avec ou sans `:` devant ; `None` pour un mot inconnu ou des arguments en trop.
pub fn parse_command(input: &str) -> Option<Intent> {
    let word = input.trim().trim_start_matches(':').trim();
    COMMANDS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(word))
        .map(|(_, intent)| *intent)
}
