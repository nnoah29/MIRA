//! Méthodes d'authentification et références de secrets (jamais de valeur de secret).

use serde::{Deserialize, Serialize};

use crate::ids::kinds::{AccountId, ProfileId};

/// Famille de méthode d'authentification d'un compte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethodKind {
    #[serde(rename = "oauth2")]
    OAuth2,
    AppPassword,
    Password,
}

/// Paramètres OAuth2 d'un provider (adresses publiques et portées demandées).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OAuthSettings {
    pub authorize_url: String,
    pub token_url: String,
    pub scopes: Vec<String>,
}

/// Méthode d'authentification proposée par un provider, avec ses paramètres.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthMethod {
    #[serde(rename = "oauth2")]
    OAuth2(OAuthSettings),
    AppPassword,
    Password,
}

impl AuthMethod {
    /// Donne la famille de cette méthode.
    pub fn kind(&self) -> AuthMethodKind {
        match self {
            Self::OAuth2(_) => AuthMethodKind::OAuth2,
            Self::AppPassword => AuthMethodKind::AppPassword,
            Self::Password => AuthMethodKind::Password,
        }
    }
}

/// Nature d'un secret conservé dans le trousseau.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretKind {
    Password,
    AccessToken,
    RefreshToken,
}

/// Référence à un secret du trousseau : identifie où il vit, sans jamais le contenir.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SecretRef {
    pub profile: ProfileId,
    pub account: AccountId,
    pub kind: SecretKind,
}

impl SecretRef {
    /// Donne la clé de stockage `mira/<profil>/<compte>/<nature>` (D-10, 7.7).
    pub fn storage_key(&self) -> String {
        let kind = match self.kind {
            SecretKind::Password => "password",
            SecretKind::AccessToken => "access_token",
            SecretKind::RefreshToken => "refresh_token",
        };
        format!("mira/{}/{}/{kind}", self.profile, self.account)
    }
}
