//! Réglages que le moteur applique lui-même : source de vérité unique, partagée avec l'application.

use serde::{Deserialize, Serialize};

use crate::ids::kinds::AccountId;

/// Plus petit relevé périodique accepté, en secondes.
pub const MIN_POLL_SECS: u32 = 1;
/// Plus grand relevé périodique accepté, en secondes.
pub const MAX_POLL_SECS: u32 = 3600;

/// Heures calmes, en heures pleines (0 à 23) ; la plage peut passer minuit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuietHours {
    pub from: u8,
    pub to: u8,
}

/// Réglages des notifications de nouveaux messages ; les défauts sont ceux de l'app avant la phase 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub preview: bool,
    pub sound: bool,
    pub when_focused: bool,
}

impl Default for NotificationSettings {
    /// Notifier, avec aperçu et son, sauf quand MIRA est au premier plan.
    fn default() -> Self {
        Self {
            enabled: true,
            preview: true,
            sound: true,
            when_focused: false,
        }
    }
}

/// Réglages du moteur ; une valeur absente laisse le choix au moteur (délai du provider pour le relevé).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EngineSettings {
    #[serde(default)]
    pub poll_secs: Option<u32>,
    #[serde(default)]
    pub muted_accounts: Vec<AccountId>,
    #[serde(default)]
    pub quiet_hours: Option<QuietHours>,
    #[serde(default)]
    pub notifications: NotificationSettings,
}

impl EngineSettings {
    /// Noms des réglages qui diffèrent de `other`, dans un ordre stable
    pub fn changed_keys(&self, other: &Self) -> Vec<String> {
        let checks = [
            ("poll_secs", self.poll_secs != other.poll_secs),
            (
                "muted_accounts",
                self.muted_accounts != other.muted_accounts,
            ),
            ("quiet_hours", self.quiet_hours != other.quiet_hours),
            ("notifications", self.notifications != other.notifications),
        ];
        let differing = checks.into_iter().filter(|(_, differs)| *differs);
        differing.map(|(key, _)| key.to_string()).collect()
    }

    /// Vrai si les nouveaux messages de ce compte ne doivent pas être notifiés
    pub fn mutes(&self, account: &AccountId) -> bool {
        self.muted_accounts.contains(account)
    }
}
