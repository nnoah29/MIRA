#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat 1.4 : enregistrer, lister et lire les conversations de l'assistant IA.

mod support_ai_conversations;
mod support_api;

use mira_vocabulary::Timestamp;
use support_ai_conversations::{get, list, save, start};

/// Ce qui est enregistré se relit à l'identique, avec les dates du moteur
#[tokio::test]
async fn save_then_get_returns_same_payload() {
    let f = start();
    let payload = "{\"messages\":[{\"text\":\"é 😀\"}]}";
    let meta = save(&f.engine, "c-1", "Ma question", payload)
        .await
        .unwrap();
    assert_eq!(meta.created_at, Timestamp::from_millis(1_000));
    let found = get(&f.engine, "c-1").await.unwrap();
    assert_eq!(found.meta, meta);
    assert_eq!(found.payload, payload);
}

/// Enregistrer deux fois le même identifiant met la ligne à jour sans en créer une seconde
#[tokio::test]
async fn save_twice_updates_in_place() {
    let f = start();
    save(&f.engine, "c-1", "Avant", "{}").await.unwrap();
    f.clock.advance_millis(500);
    let meta = save(&f.engine, "c-1", "Après", "{\"a\":1}").await.unwrap();
    assert_eq!(meta.created_at, Timestamp::from_millis(1_000));
    assert_eq!(meta.updated_at, Timestamp::from_millis(1_500));
    assert_eq!(list(&f.engine, 50, None).await.unwrap().len(), 1);
}

/// La liste va du plus récent au plus ancien et ne porte aucun contenu
#[tokio::test]
async fn list_is_newest_first_without_payload() {
    let f = start();
    for id in ["a", "b", "c"] {
        save(&f.engine, id, id, "{\"secret\":\"CONTENU-PRIVE\"}")
            .await
            .unwrap();
        f.clock.advance_millis(10);
    }
    let items = list(&f.engine, 50, None).await.unwrap();
    let ids: Vec<&str> = items.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, ["c", "b", "a"]);
    let text = serde_json::to_string(&items).unwrap();
    assert!(!text.contains("CONTENU-PRIVE"), "{text}");
}

/// Lire une conversation inconnue donne `not_found.ai_conversation`
#[tokio::test]
async fn get_unknown_is_not_found() {
    let f = start();
    let refused = get(&f.engine, "absente").await;
    assert_eq!(refused.unwrap_err(), "not_found.ai_conversation");
}
