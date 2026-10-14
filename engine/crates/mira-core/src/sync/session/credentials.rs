//! Point d'extension vers le module d'authentification : identifiants IMAP et rafraîchissement.

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use mira_vocabulary::{Account, MiraError};

use crate::imap::ImapLogin;

/// Futur boxé, pour que les traits ci-dessous restent utilisables derrière `Arc<dyn …>`.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Fournit les identifiants d'un compte sans que la synchronisation connaisse le trousseau.
pub trait CredentialProvider: Debug + Send + Sync {
    /// Identifiants IMAP à présenter au serveur (mot de passe ou jeton d'accès).
    fn imap_login<'a>(
        &'a self,
        account: &'a Account,
    ) -> BoxFuture<'a, Result<ImapLogin, MiraError>>;

    /// Tente de renouveler les identifiants ; rend `true` si de nouveaux sont disponibles.
    fn refresh<'a>(&'a self, account: &'a Account) -> BoxFuture<'a, Result<bool, MiraError>>;
}
