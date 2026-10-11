//! Contexte partagé par toutes les sessions d'un serveur.

use crate::credentials::Credentials;
use crate::imap::capabilities::CapabilitySet;
use crate::imap::store::FakeMailStore;

/// Magasin, identifiants et extensions d'un serveur.
#[derive(Debug)]
pub(crate) struct ServerContext {
    pub store: FakeMailStore,
    pub credentials: Credentials,
    pub caps: CapabilitySet,
}
