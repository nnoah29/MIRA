//! Ressources réelles de la machine : `/proc/meminfo` et `statvfs`, sans code non sûr.

use std::path::Path;

use super::resources::SystemResources;

/// Lit les ressources du système Linux ; ailleurs, tout est inconnu.
#[derive(Debug, Clone, Copy, Default)]
pub struct LinuxResources;

/// Extrait `MemAvailable` (en kio) du texte de `/proc/meminfo` et le rend en Mio.
pub fn parse_mem_available(meminfo: &str) -> Option<u64> {
    let line = meminfo.lines().find(|l| l.starts_with("MemAvailable:"))?;
    let kib: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kib / 1024)
}

impl SystemResources for LinuxResources {
    /// Lit la mémoire disponible dans `/proc/meminfo`
    fn available_memory_mib(&self) -> Option<u64> {
        let text = std::fs::read_to_string("/proc/meminfo").ok()?;
        parse_mem_available(&text)
    }

    /// Rend l'espace utilisable par un utilisateur ordinaire sur le volume de `path`
    fn free_disk_bytes(&self, path: &Path) -> Option<u64> {
        disk_free(path)
    }
}

/// Espace libre du volume via `statvfs`.
#[cfg(unix)]
fn disk_free(path: &Path) -> Option<u64> {
    let stat = rustix::fs::statvfs(path).ok()?;
    Some(stat.f_bavail.saturating_mul(stat.f_frsize))
}

/// Inconnu hors Unix.
#[cfg(not(unix))]
fn disk_free(_path: &Path) -> Option<u64> {
    None
}
