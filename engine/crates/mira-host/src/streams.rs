//! Flux et lectures du moteur d'un hôte : événements, génération d'IA, HTML d'origine.

use crate::{Host, chat_feed, events_feed, original_html};

impl Host {
    /// Envoie chaque événement (JSON) à `emit` ; sans `after`, seuls les événements futurs sont livrés.
    pub async fn pump_events(&self, after: Option<u64>, emit: impl FnMut(String) -> bool) {
        events_feed::pump_events(self, after, emit).await
    }

    /// Envoie `started`, puis chaque delta (JSON) à `emit`, ou une seule ligne `error` ; si `emit` rend faux, annule.
    pub async fn pump_chat(&self, request_json: &str, emit: impl FnMut(String) -> bool) {
        chat_feed::pump_chat(self, request_json, emit).await
    }

    /// HTML d'origine d'un message, tel que reçu : `{html}` ou `{html: null}`.
    pub async fn original_html(&self, message: &str) -> String {
        original_html::fetch(self, message).await
    }
}
