#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « template » par la façade : les lectures n'émettent rien, le contenu ne fuit pas.

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_template_api;
mod support_write;

use mira_core::api::TemplateRequest;
use mira_vocabulary::RenderContext;
use support_api::queries::event_kinds;
use support_imap::within;
use support_template_api::calls::{call, changed_count, create, render, templates};
use support_write::fixture::Fixture;

/// Lister, lire et rendre n'émettent aucun événement
#[tokio::test]
async fn reads_emit_no_event() {
    within(async {
        let f = Fixture::start().await;
        let made = create(f.engine(), "Merci", "Corps", "k-1").await.unwrap();
        templates(f.engine()).await;
        let get = TemplateRequest::GetTemplate {
            template: made.id.clone(),
        };
        call(f.engine(), get).await.unwrap();
        render(f.engine(), &made.id, RenderContext::default())
            .await
            .unwrap();
        assert_eq!(changed_count(f.engine()).await, 1);
    })
    .await;
}

/// Le corps d'un modèle n'apparaît ni dans les erreurs ni dans les événements
#[tokio::test]
async fn template_content_never_appears_in_errors_or_events() {
    within(async {
        let f = Fixture::start().await;
        let secret = "SECRET-MARK-42";
        let made = create(f.engine(), "Merci", secret, "k-1").await.unwrap();
        let empty = create(f.engine(), "   ", secret, "k-2").await.unwrap_err();
        let taken = create(f.engine(), "merci", secret, "k-3")
            .await
            .unwrap_err();
        let long = create(f.engine(), &"n".repeat(101), secret, "k-4").await;
        let kinds = format!("{:?}", event_kinds(f.engine()).await);
        for text in [empty, taken, long.unwrap_err(), kinds] {
            assert!(!text.contains(secret), "fuite dans : {text}");
        }
        assert_eq!(made.body, secret);
    })
    .await;
}
