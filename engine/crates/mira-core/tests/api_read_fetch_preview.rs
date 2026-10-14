#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Le téléchargement à la demande range le même résumé que la synchronisation : aperçu compris.

mod support;
mod support_api;
mod support_mirror;
mod support_read;

use mira_core::operations::OperationStatus;
use support_read::mail;
use support_read::world::start_world;

/// Après un téléchargement à la demande, l'aperçu de la liste reprend le début du texte
#[tokio::test]
async fn regression_fetch_on_demand_fills_the_preview() {
    let world = start_world().await;
    let raw = mail::plain("apercu", "Le debut du corps   parle   d'un narval.");
    let id = world.deliver_header_only(&raw);
    assert_eq!(world.fetch_body(&id).await, OperationStatus::Finished);
    let stored = world
        .mirror
        .read(|tx| tx.get_message(&id))
        .unwrap()
        .unwrap();
    assert!(
        stored
            .preview
            .starts_with("Le debut du corps parle d'un narval")
    );
}
