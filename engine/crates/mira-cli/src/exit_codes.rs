//! Codes de sortie du processus, un par catégorie d'erreur du contrat (12.6).

use mira_assembly::mira_vocabulary::ErrorCategory;

/// Succès.
pub const SUCCESS: i32 = 0;
/// Entrée invalide.
pub const INVALID_INPUT: i32 = 10;
/// Élément introuvable.
pub const NOT_FOUND: i32 = 11;
/// État incompatible avec l'opération.
pub const INCOMPATIBLE_STATE: i32 = 12;
/// Erreur réseau.
pub const NETWORK: i32 = 13;
/// Erreur TLS.
pub const TLS: i32 = 14;
/// Authentification refusée.
pub const AUTHENTICATION: i32 = 15;
/// Erreur du serveur distant.
pub const SERVER: i32 = 16;
/// Erreur de stockage local.
pub const LOCAL_STORAGE: i32 = 17;
/// Erreur du trousseau de secrets.
pub const SECRETS: i32 = 18;
/// Opération annulée.
pub const CANCELLED: i32 = 19;
/// Erreur interne.
pub const INTERNAL: i32 = 84;

/// Donne le code de sortie d'une catégorie d'erreur.
pub fn exit_code_for(category: ErrorCategory) -> i32 {
    match category {
        ErrorCategory::InvalidInput => INVALID_INPUT,
        ErrorCategory::NotFound => NOT_FOUND,
        ErrorCategory::IncompatibleState => INCOMPATIBLE_STATE,
        ErrorCategory::Network => NETWORK,
        ErrorCategory::Tls => TLS,
        ErrorCategory::Authentication => AUTHENTICATION,
        ErrorCategory::Server => SERVER,
        ErrorCategory::LocalStorage => LOCAL_STORAGE,
        ErrorCategory::Secrets => SECRETS,
        ErrorCategory::Cancelled => CANCELLED,
        ErrorCategory::Internal => INTERNAL,
    }
}
