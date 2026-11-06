//! Flux et lectures du moteur d'un hôte : événements, génération d'IA, HTML d'origine.

use std::future::{Future, ready};

use crate::{Host, chat_feed, events_feed, original_html};

impl Host {
    /// Envoie chaque événement (JSON) à `emit` ; sans `after`, seuls les événements futurs sont livrés.
    pub async fn pump_events(&self, after: Option<u64>, mut emit: impl FnMut(String) -> bool) {
        events_feed::pump_events(self, after, |line| ready(emit(line))).await
    }

    /// Comme `pump_events`, avec un `emit` asynchrone : il peut attendre (contre-pression) avant de rendre la main.
    pub async fn pump_events_async<F, Fut>(&self, after: Option<u64>, emit: F)
    where
        F: FnMut(String) -> Fut,
        Fut: Future<Output = bool>,
    {
        events_feed::pump_events(self, after, emit).await
    }

    /// Envoie `started`, puis chaque delta (JSON) à `emit`, ou une seule ligne `error` ; si `emit` rend faux, annule.
    pub async fn pump_chat(&self, request_json: &str, mut emit: impl FnMut(String) -> bool) {
        chat_feed::pump_chat(self, request_json, |line| ready(emit(line))).await
    }

    /// Comme `pump_chat`, avec un `emit` asynchrone : tant qu'il attend, la génération n'est plus lue.
    pub async fn pump_chat_async<F, Fut>(&self, request_json: &str, emit: F)
    where
        F: FnMut(String) -> Fut,
        Fut: Future<Output = bool>,
    {
        chat_feed::pump_chat(self, request_json, emit).await
    }

    /// HTML d'origine d'un message, tel que reçu : `{html}` ou `{html: null}`.
    pub async fn original_html(&self, message: &str) -> String {
        original_html::fetch(self, message).await
    }
}
