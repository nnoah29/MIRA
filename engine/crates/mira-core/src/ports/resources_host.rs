//! Ressources réelles de la machine, sans code non sûr : `/proc/meminfo` et `statvfs` sous Unix, `sysinfo` sous Windows.

use std::path::Path;

use super::host_probe::{disk_free, memory_available};
use super::resources::SystemResources;

/// Lit les ressources du système hôte ; ce qui ne peut pas l'être reste inconnu.
#[derive(Debug, Clone, Copy, Default)]
pub struct HostResources;

/// Extrait `MemAvailable` (en kio) du texte de `/proc/meminfo` et le rend en Mio.
pub fn parse_mem_available(meminfo: &str) -> Option<u64> {
    let line = meminfo.lines().find(|l| l.starts_with("MemAvailable:"))?;
    let kib: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kib / 1024)
}

impl SystemResources for HostResources {
    /// Rend la mémoire disponible en Mio
    fn available_memory_mib(&self) -> Option<u64> {
        memory_available()
    }

    /// Rend l'espace utilisable par un utilisateur ordinaire sur le volume de `path`
    fn free_disk_bytes(&self, path: &Path) -> Option<u64> {
        disk_free(path)
    }
}
