//! Lecture des réglages du moteur.

use std::collections::HashMap;

use mira_vocabulary::{AccountId, EngineSettings, MiraError, NotificationSettings, QuietHours};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

/// Clé du relevé périodique
pub(super) const POLL_SECS: &str = "poll_secs";
/// Clé des comptes en sourdine (liste JSON)
pub(super) const MUTED_ACCOUNTS: &str = "muted_accounts";
/// Clé des heures calmes (`de-à`)
pub(super) const QUIET_HOURS: &str = "quiet_hours";
/// Clé des réglages de notification (objet JSON ; absente tant que les défauts s'appliquent)
pub(super) const NOTIFICATIONS: &str = "notifications";

/// Heures calmes écrites `de-à` ; une valeur illisible est ignorée
fn parse_quiet(text: &str) -> Option<QuietHours> {
    let (from, to) = text.split_once('-')?;
    Some(QuietHours {
        from: from.parse().ok()?,
        to: to.parse().ok()?,
    })
}

/// Comptes en sourdine lus d'une liste JSON ; une valeur illisible est ignorée
fn parse_muted(text: &str) -> Vec<AccountId> {
    let ids: Vec<String> = serde_json::from_str(text).unwrap_or_default();
    ids.into_iter().map(AccountId::from_raw).collect()
}

/// Réglages de notification lus d'un objet JSON ; une valeur illisible rend les défauts
fn parse_notifications(text: &str) -> NotificationSettings {
    serde_json::from_str(text).unwrap_or_default()
}

impl Tx<'_> {
    /// Réglages enregistrés ; un réglage jamais renseigné garde sa valeur par défaut
    pub fn get_engine_settings(&self) -> Result<EngineSettings, MiraError> {
        let mut statement = self
            .sql()
            .prepare("SELECT key, value FROM engine_settings")
            .map_err(storage_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(storage_error)?;
        let stored: HashMap<String, String> =
            rows.collect::<Result<_, _>>().map_err(storage_error)?;
        Ok(EngineSettings {
            poll_secs: stored.get(POLL_SECS).and_then(|text| text.parse().ok()),
            muted_accounts: stored
                .get(MUTED_ACCOUNTS)
                .map_or_else(Vec::new, |t| parse_muted(t)),
            quiet_hours: stored.get(QUIET_HOURS).and_then(|text| parse_quiet(text)),
            notifications: stored
                .get(NOTIFICATIONS)
                .map_or_else(NotificationSettings::default, |t| parse_notifications(t)),
        })
    }
}
