#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Lecture de la mémoire disponible dans `/proc/meminfo` et ressources fixées des tests.

use std::path::Path;

use mira_core::ports::resources_host::parse_mem_available;
use mira_core::ports::{FixedResources, SystemResources};

/// `MemAvailable` en kio est rendu en Mio
#[test]
fn parse_mem_available_converts_kib_to_mib() {
    let text = "MemTotal: 16000000 kB\nMemFree: 100 kB\nMemAvailable:    5242880 kB\n";
    assert_eq!(parse_mem_available(text), Some(5120));
}

/// Un texte sans la ligne ou illisible donne `None`
#[test]
fn parse_mem_available_is_none_without_the_line() {
    assert_eq!(parse_mem_available("MemTotal: 1 kB\n"), None);
    assert_eq!(parse_mem_available("MemAvailable: abc kB\n"), None);
}

/// Les ressources fixées rendent exactement ce qu'on leur donne
#[test]
fn fixed_resources_return_their_values() {
    let res = FixedResources::new(Some(123), None);
    assert_eq!(res.available_memory_mib(), Some(123));
    assert_eq!(res.free_disk_bytes(Path::new("/")), None);
}
