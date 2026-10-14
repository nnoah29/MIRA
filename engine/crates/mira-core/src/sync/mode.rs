//! Mode de la synchronisation en arrière-plan d'un moteur : absente, ou active avec ses délais.

use super::supervisor::SupervisorConfig;

/// Le moteur synchronise-t-il les comptes en arrière-plan ? Absent par défaut ; l'assemblage de production l'active.
#[derive(Debug, Clone, Copy, Default)]
pub enum BackgroundSync {
    #[default]
    Off,
    On(SupervisorConfig),
}
