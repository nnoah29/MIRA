//! État de l'assistant d'IA porté par le moteur : connexions par navigateur, sessions, modèle local.

use super::key_login::AiKeyLogins;
use super::local::{LocalInstallSlot, LocalRuntimeManager};
use super::session::AiSessionLocks;

/// Ce que l'assistant d'IA garde en mémoire pendant la vie du moteur.
#[derive(Debug, Default)]
pub(crate) struct AiState {
    pub(crate) key_logins: AiKeyLogins,
    pub(crate) session_locks: AiSessionLocks,
    pub(crate) local_runtime: LocalRuntimeManager,
    pub(crate) local_install: LocalInstallSlot,
}
