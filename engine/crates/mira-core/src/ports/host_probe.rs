//! Lectures de la mémoire et du disque propres à chaque système.

use std::path::Path;

/// Mémoire disponible lue dans `/proc/meminfo`.
#[cfg(unix)]
pub(super) fn memory_available() -> Option<u64> {
    super::resources_host::parse_mem_available(&std::fs::read_to_string("/proc/meminfo").ok()?)
}

/// Mémoire disponible rendue par le système.
#[cfg(windows)]
pub(super) fn memory_available() -> Option<u64> {
    let mut system = sysinfo::System::new().ok()?;
    system.refresh_memory();
    Some(system.available_memory() / (1024 * 1024))
}

/// Inconnue sur les autres systèmes.
#[cfg(not(any(unix, windows)))]
pub(super) fn memory_available() -> Option<u64> {
    None
}

/// Espace libre du volume via `statvfs`.
#[cfg(unix)]
pub(super) fn disk_free(path: &Path) -> Option<u64> {
    let stat = rustix::fs::statvfs(path).ok()?;
    Some(stat.f_bavail.saturating_mul(stat.f_frsize))
}

/// Espace libre du volume dont le point de montage est le plus long préfixe de `path`.
#[cfg(windows)]
pub(super) fn disk_free(path: &Path) -> Option<u64> {
    let disks = sysinfo::Disks::new_with_refreshed_list().ok()?;
    disks
        .iter()
        .filter(|d| path.starts_with(d.mount_point()))
        .max_by_key(|d| d.mount_point().as_os_str().len())
        .map(|d| d.available_space())
}

/// Inconnu sur les autres systèmes.
#[cfg(not(any(unix, windows)))]
pub(super) fn disk_free(_path: &Path) -> Option<u64> {
    None
}
