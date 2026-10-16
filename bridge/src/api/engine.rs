//! Points d'entrée Dart : tout passe par des chaînes JSON de l'enveloppe versionnée du contrat.

use flutter_rust_bridge::frb;
use mira_assembly::mira_vocabulary::{CONTRACT_VERSION, ProfileId};
use mira_assembly::production_config;

use crate::frb_generated::StreamSink;
use crate::runtime;

/// Version du contrat compilée dans le moteur, au format `majeur.mineur`.
#[frb(sync)]
pub fn contract_version() -> String {
    format!("{}.{}", CONTRACT_VERSION.major, CONTRACT_VERSION.minor)
}

/// Ouvre le moteur sur le profil nommé (trousseau et dossier de données du système).
pub async fn open_engine(profile: String) -> String {
    runtime::open(production_config(ProfileId::from_raw(profile)))
}

/// Exécute une requête du contrat (JSON) et rend l'enveloppe JSON `{ok, data | error}`.
pub async fn execute(request_json: String) -> String {
    runtime::execute(&request_json).await
}

/// Diffuse les événements du moteur (JSON) à partir du numéro donné, jusqu'à la fermeture du flux Dart.
pub async fn watch_events(after: Option<u64>, sink: StreamSink<String>) {
    runtime::pump_events(after, move |line| sink.add(line).is_ok()).await
}

/// Ferme le moteur et libère le verrou du profil.
pub async fn close_engine() -> String {
    runtime::close().await
}
