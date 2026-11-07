//! Notifications du moteur embarqué : le notifieur du moteur décide, Dart affiche ce qu'on lui envoie.

use std::sync::Arc;
use std::time::Duration;

use mira_host::notifier::{Notification, NotificationSink, SystemClock, Texts, run_notifier};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use crate::session::host;

const WINDOW: Duration = Duration::from_secs(2);

/// Récepteur qui pousse chaque notification, en JSON, dans un canal lu par le flux Dart.
#[derive(Debug)]
struct ChannelSink(UnboundedSender<String>);

impl NotificationSink for ChannelSink {
    /// Envoie la notification ; un flux fermé la perd sans bruit.
    fn show(&self, notification: Notification) {
        let _ = self.0.send(notification_json(&notification));
    }
}

/// Notification en JSON plat : `id`, `title`, `body`, `sound`, `message` (message à ouvrir, ou `null`).
pub fn notification_json(notification: &Notification) -> String {
    serde_json::json!({
        "id": notification.id,
        "title": notification.title,
        "body": notification.body,
        "sound": notification.sound,
        "message": notification.message,
    })
    .to_string()
}

/// Lance le notifieur et envoie chaque notification (JSON) à `emit` ; si `emit` rend faux ou si le moteur se ferme, le notifieur s'arrête.
pub async fn pump(locale: &str, mut emit: impl FnMut(String) -> bool) {
    let (sender, mut lines) = unbounded_channel();
    let task = tokio::spawn(run_notifier(
        host(),
        Arc::new(ChannelSink(sender)),
        Arc::new(SystemClock),
        Texts::for_locale(locale),
        WINDOW,
    ));
    while let Some(line) = lines.recv().await {
        if !emit(line) {
            break;
        }
    }
    task.abort();
}
