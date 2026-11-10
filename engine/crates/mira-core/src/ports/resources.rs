//! Ressources de la machine (mémoire disponible, espace disque libre) : injectables pour les tests.

use std::fmt::Debug;
use std::path::Path;

/// Ce que le moteur peut demander à la machine avant d'installer ou de lancer un modèle.
pub trait SystemResources: Send + Sync + Debug {
    /// Mémoire vive disponible en Mio, si elle est connue
    fn available_memory_mib(&self) -> Option<u64>;

    /// Espace libre en octets sur le volume qui porte `path`, s'il est connu
    fn free_disk_bytes(&self, path: &Path) -> Option<u64>;
}

/// Ressources fixées à la main, pour des tests déterministes.
#[derive(Debug, Clone, Copy)]
pub struct FixedResources {
    memory_mib: Option<u64>,
    disk_bytes: Option<u64>,
}

impl FixedResources {
    /// Crée des ressources aux valeurs données
    pub fn new(memory_mib: Option<u64>, disk_bytes: Option<u64>) -> Self {
        Self {
            memory_mib,
            disk_bytes,
        }
    }
}

impl SystemResources for FixedResources {
    /// Rend la mémoire fixée
    fn available_memory_mib(&self) -> Option<u64> {
        self.memory_mib
    }

    /// Rend l'espace disque fixé, quel que soit le chemin
    fn free_disk_bytes(&self, _path: &Path) -> Option<u64> {
        self.disk_bytes
    }
}
