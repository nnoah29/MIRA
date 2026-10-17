//! Contexte partagé par les connexions HTTP : état OAuth et identifiants.

use super::state::OAuthShared;
use crate::credentials::Credentials;

/// État du serveur d'autorisation et registre d'identifiants des faux serveurs.
#[derive(Clone)]
pub(crate) struct OAuthContext {
    pub(crate) shared: OAuthShared,
    pub(crate) credentials: Credentials,
}
