//! Masquage de départ d'une connexion d'IA selon sa destination : coupé vers la boucle locale, actif ailleurs.

use mira_vocabulary::MaskingSettings;

use crate::ai::access::is_loopback_url;

/// Réglage par défaut de l'adresse choisie ou, à défaut, de celle du fournisseur ; illisible => distant
pub(super) fn default_masking(provider_url: &str, chosen: Option<&str>) -> MaskingSettings {
    match is_loopback_url(chosen.unwrap_or(provider_url)) {
        Ok(true) => MaskingSettings::local_default(),
        _ => MaskingSettings::remote_default(),
    }
}
