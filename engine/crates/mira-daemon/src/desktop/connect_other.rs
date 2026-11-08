//! Connexion au bureau hors Linux : pas de bus ni d'icône tant que les adaptateurs du système ne sont pas branchés.

use std::sync::Arc;

use mira_host::notifier::Texts;

use super::Desktop;
use crate::server::ServerContext;

impl Desktop {
    /// Sans adaptateur du système, le démon continue sans notification ni icône.
    pub async fn connect(_ctx: &Arc<ServerContext>) -> Self {
        Self::new(None, Texts::for_locale("en"))
    }
}
