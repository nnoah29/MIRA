//! Simule le clic d'un utilisateur sur une notification du faux service.

/// Simule un clic sur l'action `key` de la notification `id`.
pub async fn click(connection: &zbus::Connection, id: u32, key: &str) {
    connection
        .emit_signal(
            None::<&str>,
            "/org/freedesktop/Notifications",
            "org.freedesktop.Notifications",
            "ActionInvoked",
            &(id, key),
        )
        .await
        .unwrap();
}
