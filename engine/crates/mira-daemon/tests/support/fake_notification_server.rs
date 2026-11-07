//! Faux service `org.freedesktop.Notifications` : garde ce qu'on lui demande d'afficher et peut simuler un clic.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use zbus::zvariant::OwnedValue;

/// Une notification reçue par le faux service.
#[derive(Debug, Clone)]
pub struct Seen {
    pub id: u32,
    pub summary: String,
    pub body: String,
    pub actions: Vec<String>,
    pub hint_keys: Vec<String>,
}

/// Objet exposé sur le bus : numérote les notifications et les garde.
pub struct FakeNotifications {
    seen: Arc<Mutex<Vec<Seen>>>,
    next: AtomicU32,
    capabilities: Vec<String>,
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl FakeNotifications {
    /// Reçoit une notification et rend son numéro.
    #[allow(clippy::too_many_arguments)]
    fn notify(
        &self,
        _app: String,
        _replaces: u32,
        _icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, OwnedValue>,
        _timeout: i32,
    ) -> u32 {
        let id = self.next.fetch_add(1, Ordering::SeqCst);
        let mut hint_keys: Vec<String> = hints.keys().cloned().collect();
        hint_keys.sort();
        let entry = Seen {
            id,
            summary,
            body,
            actions,
            hint_keys,
        };
        self.seen.lock().unwrap().push(entry);
        id
    }

    /// Capacités annoncées.
    fn get_capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }

    /// Identité du faux service.
    fn get_server_information(&self) -> (String, String, String, String) {
        ("fake".into(), "tests".into(), "1".into(), "1.2".into())
    }

    /// Fermeture demandée : sans effet.
    fn close_notification(&self, _id: u32) {}
}

/// Démarre le faux service sans balisage dans le corps ; rend la connexion (à garder) et la liste des notifications reçues.
pub async fn serve(address: &str) -> (zbus::Connection, Arc<Mutex<Vec<Seen>>>) {
    serve_with(address, &["actions", "body"]).await
}

/// Démarre le faux service annonçant ces capacités (par exemple `body-markup`).
pub async fn serve_with(
    address: &str,
    capabilities: &[&str],
) -> (zbus::Connection, Arc<Mutex<Vec<Seen>>>) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let object = FakeNotifications {
        seen: seen.clone(),
        next: AtomicU32::new(1),
        capabilities: capabilities.iter().map(|c| c.to_string()).collect(),
    };
    let connection = zbus::connection::Builder::address(address)
        .unwrap()
        .name("org.freedesktop.Notifications")
        .unwrap()
        .serve_at("/org/freedesktop/Notifications", object)
        .unwrap()
        .build()
        .await
        .unwrap();
    (connection, seen)
}
