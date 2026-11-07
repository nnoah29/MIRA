//! Faux hôte de zone de notification (`org.kde.StatusNotifierWatcher`) : enregistre les icônes qui s'annoncent.

use std::sync::{Arc, Mutex};

/// Objet exposé sur le bus : garde les services enregistrés.
pub struct FakeWatcher {
    items: Arc<Mutex<Vec<String>>>,
}

#[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
impl FakeWatcher {
    /// Une icône s'enregistre.
    fn register_status_notifier_item(&self, service: String) {
        self.items.lock().unwrap().push(service);
    }

    /// Un hôte s'enregistre : sans effet.
    fn register_status_notifier_host(&self, _service: String) {}

    /// Un hôte est présent.
    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        true
    }

    /// Version du protocole.
    #[zbus(property)]
    fn protocol_version(&self) -> i32 {
        0
    }

    /// Icônes enregistrées.
    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        self.items.lock().unwrap().clone()
    }
}

/// Démarre le faux hôte sur ce bus ; rend la connexion (à garder) et la liste des icônes enregistrées.
pub async fn serve_watcher(address: &str) -> (zbus::Connection, Arc<Mutex<Vec<String>>>) {
    let items = Arc::new(Mutex::new(Vec::new()));
    let connection = zbus::connection::Builder::address(address)
        .unwrap()
        .name("org.kde.StatusNotifierWatcher")
        .unwrap()
        .serve_at(
            "/StatusNotifierWatcher",
            FakeWatcher {
                items: items.clone(),
            },
        )
        .unwrap()
        .build()
        .await
        .unwrap();
    (connection, items)
}
