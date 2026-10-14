//! Dépendances du rejeu : file locale, compte, particularités du service, identifiants et serveur SMTP.

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use mira_vocabulary::{AccountId, MiraError, ProviderQuirks, ServerEndpoint};

use crate::actions::QueueContext;
use crate::smtp::SmtpLogin;

/// Futur boxé, pour que les identifiants restent injectables derrière un `dyn`.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Accès aux identifiants du compte pendant le rejeu, avec rafraîchissement silencieux du jeton (7.7, S10).
pub trait ReplayCredentials: Send + Sync + Debug {
    /// Identifiants SMTP actuels du compte
    fn smtp_login(&self, account: &AccountId) -> Result<SmtpLogin, MiraError>;

    /// Rafraîchit le jeton d'accès s'il est expiré ; sans effet pour un mot de passe
    fn renew_if_expired<'a>(
        &'a self,
        account: &'a AccountId,
    ) -> BoxFuture<'a, Result<(), MiraError>>;

    /// Rafraîchit le jeton d'accès sans condition, après un refus du serveur
    fn renew_now<'a>(&'a self, account: &'a AccountId) -> BoxFuture<'a, Result<(), MiraError>>;
}

/// Tout ce dont le rejeu d'un compte a besoin en plus de la session IMAP.
#[derive(Debug, Clone, Copy)]
pub struct ReplayContext<'a> {
    pub queue: QueueContext<'a>,
    pub account: &'a AccountId,
    pub quirks: &'a ProviderQuirks,
    pub credentials: &'a dyn ReplayCredentials,
    pub smtp: Option<&'a ServerEndpoint>,
}
