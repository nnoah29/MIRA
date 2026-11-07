//! Logique du pont, sans dépendance à flutter_rust_bridge : ouvrir, exécuter, observer, fermer.

use mira_assembly::api::EngineConfig;

use crate::session::host;

/// Ouvre le moteur sur cette configuration et rend l'enveloppe JSON du résultat.
pub fn open(config: EngineConfig) -> String {
    host().open(config)
}

/// Exécute une requête du contrat donnée en JSON et rend l'enveloppe JSON de la réponse.
pub async fn execute(request_json: &str) -> String {
    host().execute(request_json).await
}

/// Ferme le moteur en laissant quelques secondes aux synchronisations ; rend l'enveloppe du résultat.
pub async fn close() -> String {
    host().close().await
}

/// Envoie chaque événement (JSON) à `emit` ; sans `after`, seuls les événements futurs sont livrés.
pub async fn pump_events(after: Option<u64>, emit: impl FnMut(String) -> bool) {
    host().pump_events(after, emit).await
}

/// Envoie `started`, puis chaque delta (JSON) à `emit`, ou une seule ligne `error` ; si `emit` rend faux, annule.
pub async fn pump_chat(request_json: &str, emit: impl FnMut(String) -> bool) {
    host().pump_chat(request_json, emit).await
}
