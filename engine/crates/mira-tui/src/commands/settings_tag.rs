//! Étiquettes des requêtes de la vue Paramètres : chacune porte le jeton de la demande attendue.

/// Demande de la vue Paramètres à laquelle une réponse se rattache ; une réponse au jeton périmé est ignorée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTag {
    Identities(u64),
    Events(u64),
    Stats(u64),
    Operation(u64),
}
