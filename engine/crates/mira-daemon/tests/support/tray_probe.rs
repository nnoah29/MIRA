//! Lecture de l'icône d'un démon depuis le bus : libellés du menu, clic gauche, clic sur une entrée.

use std::collections::HashMap;

use zbus::zvariant::OwnedValue;

/// Mandataire D-Bus vers l'icône enregistrée.
async fn proxy<'a>(
    connection: &'a zbus::Connection,
    service: &'a str,
    path: &'a str,
    interface: &'a str,
) -> zbus::Proxy<'a> {
    zbus::Proxy::new(connection, service, path, interface)
        .await
        .unwrap()
}

/// Clic gauche sur l'icône.
pub async fn activate(connection: &zbus::Connection, service: &str) {
    let item = proxy(
        connection,
        service,
        "/StatusNotifierItem",
        "org.kde.StatusNotifierItem",
    )
    .await;
    item.call_method("Activate", &(0i32, 0i32)).await.unwrap();
}

/// Libellés et état d'activation des trois entrées du menu, dans l'ordre.
pub async fn menu_entries(connection: &zbus::Connection, service: &str) -> Vec<(String, bool)> {
    let menu = proxy(connection, service, "/MenuBar", "com.canonical.dbusmenu").await;
    let ids = vec![1i32, 2, 3];
    let names = vec!["label".to_string(), "enabled".to_string()];
    let reply = menu
        .call_method("GetGroupProperties", &(ids, names))
        .await
        .unwrap();
    let groups: Vec<(i32, HashMap<String, OwnedValue>)> = reply.body().deserialize().unwrap();
    groups
        .into_iter()
        .map(|(_, props)| {
            let label: String = props["label"].clone().try_into().unwrap();
            let enabled = props
                .get("enabled")
                .is_none_or(|v| bool::try_from(v.clone()).unwrap());
            (label, enabled)
        })
        .collect()
}

/// Clique sur l'entrée de menu numéro `id`.
pub async fn click_entry(connection: &zbus::Connection, service: &str, id: i32) {
    let menu = proxy(connection, service, "/MenuBar", "com.canonical.dbusmenu").await;
    let data = zbus::zvariant::Value::from(0i32);
    menu.call_method("Event", &(id, "clicked", data, 0u32))
        .await
        .unwrap();
}
