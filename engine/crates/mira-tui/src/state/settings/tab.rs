//! Onglets de la vue Paramètres.

/// Les quatre pages de la vue Paramètres.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SettingsTab {
    #[default]
    Accounts,
    Identities,
    Maintenance,
    Events,
}

impl SettingsTab {
    /// Tous les onglets, dans l'ordre d'affichage.
    pub fn all() -> [SettingsTab; 4] {
        [
            SettingsTab::Accounts,
            SettingsTab::Identities,
            SettingsTab::Maintenance,
            SettingsTab::Events,
        ]
    }

    /// Titre de l'onglet.
    pub fn title(self) -> &'static str {
        match self {
            SettingsTab::Accounts => "Comptes",
            SettingsTab::Identities => "Identités",
            SettingsTab::Maintenance => "Maintenance",
            SettingsTab::Events => "Événements",
        }
    }

    /// Onglet suivant, en boucle.
    pub fn next(self) -> SettingsTab {
        match self {
            SettingsTab::Accounts => SettingsTab::Identities,
            SettingsTab::Identities => SettingsTab::Maintenance,
            SettingsTab::Maintenance => SettingsTab::Events,
            SettingsTab::Events => SettingsTab::Accounts,
        }
    }

    /// Onglet précédent, en boucle.
    pub fn previous(self) -> SettingsTab {
        match self {
            SettingsTab::Accounts => SettingsTab::Events,
            SettingsTab::Identities => SettingsTab::Accounts,
            SettingsTab::Maintenance => SettingsTab::Identities,
            SettingsTab::Events => SettingsTab::Maintenance,
        }
    }
}
