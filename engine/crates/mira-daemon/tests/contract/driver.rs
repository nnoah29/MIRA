//! Le point d'entrée commun des scénarios de contrat : un moteur ouvert, atteint par l'un des chemins.

use mira_assembly::api::EngineConfig;

/// Un chemin vers un moteur : hôte direct, socket du démon ou pont FFI ; les scénarios n'en connaissent pas d'autre.
#[allow(async_fn_in_trait)]
pub trait Driver: Sized {
    /// Ouvre un moteur sur cette configuration et rend le chemin qui y mène ; sans configuration, l'hôte n'est jamais ouvert.
    async fn start(config: Option<EngineConfig>) -> Self;

    /// Exécute une requête du contrat (JSON) et rend l'enveloppe JSON.
    async fn execute(&self, request_json: &str) -> String;

    /// Envoie chaque événement (JSON) à `emit` ; sans `after`, seuls les événements futurs sont livrés.
    async fn pump_events(&self, after: Option<u64>, emit: impl FnMut(String) -> bool);

    /// Envoie `started`, les deltas puis rien, ou une seule ligne d'erreur ; si `emit` rend faux, annule.
    async fn pump_chat(&self, request_json: &str, emit: impl FnMut(String) -> bool);

    /// HTML d'origine d'un message : l'enveloppe `{html}`.
    async fn original_html(&self, message: &str) -> String;

    /// Ferme le moteur et rend l'enveloppe du résultat.
    async fn close(&self) -> String;
}
