//! Jetons obtenus auprès du service d'autorisation (valeurs masquées).

use mira_vocabulary::Timestamp;

use crate::ports::SecretValue;

/// Jeton d'accès, jeton de rafraîchissement éventuel et instant d'expiration du jeton d'accès.
#[derive(Debug, Clone)]
pub struct TokenSet {
    pub access_token: SecretValue,
    pub refresh_token: Option<SecretValue>,
    pub expires_at: Option<Timestamp>,
}
