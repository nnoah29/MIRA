#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat 1.4 : pagination de la liste et rétention de 200 conversations.

mod support_ai_conversations;
mod support_api;

use support_ai_conversations::{list, save, start};

/// `limit` doit valoir de 1 à 200
#[tokio::test]
async fn limit_zero_or_over_200_is_refused() {
    let f = start();
    for limit in [0, 201, u32::MAX] {
        let refused = list(&f.engine, limit, None).await;
        assert_eq!(refused.unwrap_err(), "invalid_input.parameter");
    }
    assert!(list(&f.engine, 1, None).await.unwrap().is_empty());
    assert!(list(&f.engine, 200, None).await.unwrap().is_empty());
}

/// La 201e conversation fait disparaître la plus ancienne, jamais celle qu'on enregistre
#[tokio::test]
async fn the_201st_conversation_evicts_the_oldest() {
    let f = start();
    for n in 0..=200 {
        save(&f.engine, &format!("c-{n}"), "T", "{}").await.unwrap();
        f.clock.advance_millis(1);
    }
    let items = list(&f.engine, 200, None).await.unwrap();
    assert_eq!(items.len(), 200);
    assert_eq!(items[0].id, "c-200");
    assert!(items.iter().all(|m| m.id != "c-0"));
    let page = list(&f.engine, 50, Some(&items[49].id)).await.unwrap();
    assert_eq!(page[0].id, items[50].id);
}
