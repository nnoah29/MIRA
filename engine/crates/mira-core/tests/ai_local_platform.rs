#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Nom de plateforme du descripteur selon le système et l'architecture.

use mira_core::ai::local::platform_of;

/// Linux et Windows, en 64 bits Intel/AMD et ARM, sont reconnus ; le reste ne l'est pas
#[test]
fn platform_names_follow_os_and_arch() {
    assert_eq!(platform_of("linux", "x86_64"), Some("linux-x86_64"));
    assert_eq!(platform_of("linux", "aarch64"), Some("linux-aarch64"));
    assert_eq!(platform_of("windows", "x86_64"), Some("windows-x86_64"));
    assert_eq!(platform_of("windows", "aarch64"), Some("windows-aarch64"));
    assert_eq!(platform_of("macos", "aarch64"), None);
    assert_eq!(platform_of("windows", "x86"), None);
    assert_eq!(platform_of("android", "aarch64"), None);
}
