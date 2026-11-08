//! État affiché par l'icône et son texte, français ou anglais selon la langue du système.

use mira_protocol::HostState;

/// État résumé du démon pour l'icône.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayStatus {
    Starting,
    UpToDate,
    KeyringLocked,
    UpdateRequired,
}

/// Texte de la ligne d'état.
pub fn status_line(status: TrayStatus, locale: &str) -> &'static str {
    let french = locale.starts_with("fr");
    match (status, french) {
        (TrayStatus::Starting, true) => "Démarrage…",
        (TrayStatus::UpToDate, true) => "À jour",
        (TrayStatus::KeyringLocked, true) => "Trousseau verrouillé",
        (TrayStatus::UpdateRequired, true) => "Mise à jour requise",
        (TrayStatus::Starting, false) => "Starting…",
        (TrayStatus::UpToDate, false) => "Up to date",
        (TrayStatus::KeyringLocked, false) => "Keyring locked",
        (TrayStatus::UpdateRequired, false) => "Update required",
    }
}

/// État d'icône qui correspond à l'état de l'hôte.
pub fn tray_status_of(state: HostState) -> TrayStatus {
    match state {
        HostState::Starting => TrayStatus::Starting,
        HostState::Ready => TrayStatus::UpToDate,
        HostState::KeyringLocked => TrayStatus::KeyringLocked,
        HostState::UpdateRequired => TrayStatus::UpdateRequired,
    }
}

/// Libellés « Ouvrir » et « Quitter » selon la langue.
pub fn menu_labels(locale: &str) -> (&'static str, &'static str) {
    if locale.starts_with("fr") {
        ("Ouvrir MIRA", "Quitter MIRA")
    } else {
        ("Open MIRA", "Quit MIRA")
    }
}
