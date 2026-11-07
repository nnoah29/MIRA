//! Faux Secret Service (`org.freedesktop.secrets`) : une collection par défaut dont l'état « verrouillé » est réglable.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use zbus::zvariant::OwnedObjectPath;

const DEFAULT_PATH: &str = "/org/freedesktop/secrets/collection/login";

/// Service : répond à `ReadAlias`, avec la collection par défaut ou « aucune » (`/`).
pub struct FakeService {
    has_default: bool,
}

#[zbus::interface(name = "org.freedesktop.Secret.Service")]
impl FakeService {
    /// Chemin de la collection de cet alias, ou `/` s'il n'y en a pas.
    fn read_alias(&self, _name: String) -> OwnedObjectPath {
        let path = if self.has_default { DEFAULT_PATH } else { "/" };
        OwnedObjectPath::try_from(path).unwrap()
    }
}

/// Collection par défaut : sa propriété `Locked` suit le drapeau partagé.
pub struct FakeCollection {
    locked: Arc<AtomicBool>,
}

#[zbus::interface(name = "org.freedesktop.Secret.Collection")]
impl FakeCollection {
    /// Vrai tant que la collection est verrouillée.
    #[zbus(property)]
    fn locked(&self) -> bool {
        self.locked.load(Ordering::SeqCst)
    }
}

/// Démarre le faux service sur ce bus ; rend la connexion (à garder) et le drapeau « verrouillé ».
pub async fn serve_secrets(
    address: &str,
    locked: bool,
    has_default: bool,
) -> (zbus::Connection, Arc<AtomicBool>) {
    let flag = Arc::new(AtomicBool::new(locked));
    let connection = zbus::connection::Builder::address(address)
        .unwrap()
        .name("org.freedesktop.secrets")
        .unwrap()
        .serve_at("/org/freedesktop/secrets", FakeService { has_default })
        .unwrap()
        .serve_at(
            DEFAULT_PATH,
            FakeCollection {
                locked: flag.clone(),
            },
        )
        .unwrap()
        .build()
        .await
        .unwrap();
    (connection, flag)
}
