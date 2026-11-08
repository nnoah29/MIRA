//! Choix de la sonde du trousseau selon le système et le mode des secrets.

use crate::unlock::{KeyringProbe, NeverLocked};

/// Sonde de production : le Secret Service sous Linux, sauf si les secrets sont dans un fichier chiffré.
#[cfg(target_os = "linux")]
pub fn probe_for(file_secrets: bool) -> Box<dyn KeyringProbe> {
    if file_secrets {
        Box::new(NeverLocked)
    } else {
        Box::new(crate::secret_service::SecretServiceProbe::connect())
    }
}

/// Hors Linux, le magasin du système ne se verrouille pas.
#[cfg(not(target_os = "linux"))]
pub fn probe_for(_file_secrets: bool) -> Box<dyn KeyringProbe> {
    Box::new(NeverLocked)
}
