//! Attente du signal d'arrêt du serveur.

use tokio::sync::watch;

/// Attend le signal d'arrêt ; ne garde aucune référence au canal pendant l'attente (futur `Send`).
pub async fn wait_closing(closing: &mut watch::Receiver<bool>) {
    let _ = closing.wait_for(|closing| *closing).await.map(|_| ());
}
