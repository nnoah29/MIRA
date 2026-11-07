#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! État affiché par l'icône : texte de la ligne d'état et correspondance avec l'état de l'hôte.

use mira_daemon::desktop::{TrayStatus, status_line, tray_status_of};
use mira_protocol::HostState;

/// La ligne d'état est en français pour un système français, en anglais sinon.
#[test]
fn the_status_line_follows_the_locale() {
    assert_eq!(status_line(TrayStatus::UpToDate, "fr_FR.UTF-8"), "À jour");
    assert_eq!(
        status_line(TrayStatus::KeyringLocked, "fr"),
        "Trousseau verrouillé"
    );
    assert_eq!(
        status_line(TrayStatus::UpdateRequired, "fr"),
        "Mise à jour requise"
    );
    assert_eq!(status_line(TrayStatus::Starting, "fr"), "Démarrage…");
    assert_eq!(status_line(TrayStatus::UpToDate, "en_US"), "Up to date");
    assert_eq!(status_line(TrayStatus::KeyringLocked, ""), "Keyring locked");
}

/// Chaque état de l'hôte a son état d'icône.
#[test]
fn every_host_state_has_a_tray_status() {
    assert_eq!(tray_status_of(HostState::Starting), TrayStatus::Starting);
    assert_eq!(tray_status_of(HostState::Ready), TrayStatus::UpToDate);
    assert_eq!(
        tray_status_of(HostState::KeyringLocked),
        TrayStatus::KeyringLocked
    );
    assert_eq!(
        tray_status_of(HostState::UpdateRequired),
        TrayStatus::UpdateRequired
    );
}
