//! Notifications du moteur embarqué pour Dart, qui les affiche sans appliquer aucune règle.

use crate::frb_generated::StreamSink;

/// Diffuse les notifications décidées par le moteur (JSON : `id`, `title`, `body`, `sound`, `message`), dans la langue `locale` ; fermer le flux Dart l'arrête.
pub async fn watch_notifications(locale: String, sink: StreamSink<String>) {
    crate::notifications::pump(&locale, move |line| sink.add(line).is_ok()).await
}
