//! Échanges de flux du client de test : ouverture, crédits, lecture jusqu'à la fin.

use std::time::Duration;

use mira_protocol::{EndReason, Frame};
use serde_json::{Value, json};

use super::{Recv, TestClient};

impl TestClient {
    /// Ouvre un flux d'événements et rend son identifiant.
    pub async fn open_events(&mut self, id: u64, after: Option<u64>, credit: u32) {
        self.send(Frame::OpenEvents { id, after, credit }).await;
    }

    /// Ouvre un flux de chat avec cette demande ; un texte qui n'est pas du JSON voyage comme chaîne JSON (le protocole ne porte que du JSON).
    pub async fn open_chat(&mut self, id: u64, request: &str, credit: u32) {
        let request =
            serde_json::from_str(request).unwrap_or_else(|_| Value::String(request.into()));
        self.send(Frame::OpenChat {
            id,
            request,
            credit,
        })
        .await;
    }

    /// Numéro du dernier événement du journal (0 s'il est vide), lu par le contrat.
    pub async fn latest_seq(&mut self) -> u64 {
        let request = json!({"domain": "event", "request": {"op": "get_events",
            "filter": {"accounts": [], "categories": [], "after": null}, "limit": 500}});
        let page = self.call(request).await;
        let events = page["data"]["data"].as_array().unwrap();
        events.last().map_or(0, |e| e["seq"].as_u64().unwrap())
    }

    /// Provoque un événement `local_labels_changed` : crée un libellé local.
    pub async fn make_event(&mut self, name: &str) {
        let request = json!({"domain": "local_label", "request": {
            "op": "create_local_label", "name": name, "idempotency_key": name}});
        assert_eq!(self.call(request).await["ok"], true);
    }

    /// Lit jusqu'à `end` du flux `id` ; rend les lignes reçues et la raison de fin.
    pub async fn collect_until_end(&mut self, id: u64) -> (Vec<Value>, EndReason) {
        let mut lines = Vec::new();
        loop {
            match self.recv().await {
                Some(Frame::ChatDelta { id: got, line }) if got == id => lines.push(line),
                Some(Frame::Event { id: got, event }) if got == id => lines.push(event),
                Some(Frame::End { id: got, reason }) if got == id => return (lines, reason),
                other => panic!("trame inattendue : {other:?}"),
            }
        }
    }

    /// Compte les trames de flux reçues avant `quiet` de silence.
    pub async fn count_until_quiet(&mut self, quiet: Duration) -> Vec<Frame> {
        let mut frames = Vec::new();
        while let Recv::Frame(frame) = self.recv_for(quiet).await {
            frames.push(frame);
        }
        frames
    }
}
