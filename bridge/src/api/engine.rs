//! Points d'entrée Dart : tout passe par des chaînes JSON de l'enveloppe versionnée du contrat.

use flutter_rust_bridge::frb;
use mira_assembly::mira_vocabulary::{CONTRACT_VERSION, ProfileId};
use mira_assembly::production_config;
use mira_assembly::sync::BackgroundSync;

use crate::frb_generated::StreamSink;
use crate::polling::supervisor_for;
use crate::runtime;

/// Version du contrat compilée dans le moteur, au format `majeur.mineur`.
#[frb(sync)]
pub fn contract_version() -> String {
    format!("{}.{}", CONTRACT_VERSION.major, CONTRACT_VERSION.minor)
}

/// Ouvre le moteur sur le profil nommé ; `poll_secs` fixe le relevé périodique (30 s à 1 h), sinon le délai du fournisseur.
/// `home` remplace le dossier de données du système (Android : dossier privé de l'app) ; `secrets_key`
/// (32 octets, gardés par l'app dans le Keystore) remplace le trousseau du système par un fichier chiffré dans `home`.
pub async fn open_engine(
    profile: String,
    poll_secs: Option<u32>,
    home: Option<String>,
    secrets_key: Option<Vec<u8>>,
) -> String {
    let mut config = production_config(ProfileId::from_raw(profile));
    config.network.background_sync = BackgroundSync::On(supervisor_for(poll_secs));
    match crate::platform::adapt(&mut config, home, secrets_key) {
        Ok(()) => runtime::open(config),
        Err(error) => crate::envelope::failure(&error),
    }
}

/// Exécute une requête du contrat (JSON) et rend l'enveloppe JSON `{ok, data | error}`.
pub async fn execute(request_json: String) -> String {
    runtime::execute(&request_json).await
}

/// Diffuse les événements du moteur (JSON) à partir du numéro donné, jusqu'à la fermeture du flux Dart.
pub async fn watch_events(after: Option<u64>, sink: StreamSink<String>) {
    runtime::pump_events(after, move |line| sink.add(line).is_ok()).await
}

/// Diffuse une génération d'IA (JSON `ChatStart`) en lignes JSON `started`, deltas ou `error` ; fermer le flux Dart l'annule.
pub async fn chat_stream(request_json: String, sink: StreamSink<String>) {
    runtime::pump_chat(&request_json, move |line| sink.add(line).is_ok()).await
}

/// HTML d'origine d'un message, tel que reçu (sans assainissement) : `{html}` ou `{html: null}`.
pub async fn get_original_html(message: String) -> String {
    crate::original_html::fetch(&message).await
}

/// Ferme le moteur et libère le verrou du profil.
pub async fn close_engine() -> String {
    runtime::close().await
}
