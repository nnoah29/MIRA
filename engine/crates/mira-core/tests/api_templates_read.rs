#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « template » par la façade : suppression, lecture, rendu.

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_template_api;
mod support_write;

use mira_core::api::TemplateRequest;
use mira_vocabulary::{RenderContext, TemplateId};
use support_imap::within;
use support_template_api::calls::{call, changed_count, create, render, templates};
use support_write::fixture::Fixture;

/// Supprimer retire le modèle ; le relire ou le supprimer encore donne `not_found.template`
#[tokio::test]
async fn delete_then_get_is_not_found() {
    within(async {
        let f = Fixture::start().await;
        let made = create(f.engine(), "Merci", "Corps", "k-1").await.unwrap();
        let delete = TemplateRequest::DeleteTemplate {
            template: made.id.clone(),
        };
        call(f.engine(), delete.clone()).await.unwrap();
        let get = TemplateRequest::GetTemplate { template: made.id };
        assert_eq!(
            call(f.engine(), get).await.unwrap_err(),
            "not_found.template"
        );
        assert_eq!(
            call(f.engine(), delete).await.unwrap_err(),
            "not_found.template"
        );
        assert!(templates(f.engine()).await.is_empty());
        assert_eq!(changed_count(f.engine()).await, 2);
    })
    .await;
}

/// Le rendu remplace les variables connues et signale celles restées visibles
#[tokio::test]
async fn render_replaces_variables_and_reports_unresolved() {
    within(async {
        let f = Fixture::start().await;
        let body = "Bonjour {{contact.prénom}}, le {{date}}";
        let made = create(f.engine(), "Salut", body, "k-1").await.unwrap();
        let context = RenderContext {
            date: Some("25/10/2026".to_string()),
            ..RenderContext::default()
        };
        let rendered = render(f.engine(), &made.id, context).await.unwrap();
        assert_eq!(rendered.body, "Bonjour {{contact.prénom}}, le 25/10/2026");
        assert_eq!(rendered.unresolved, ["contact.prénom"]);
        assert_eq!(rendered.subject, None);
    })
    .await;
}

/// Un modèle inconnu ne se rend pas
#[tokio::test]
async fn render_unknown_template_is_not_found() {
    within(async {
        let f = Fixture::start().await;
        let absent = TemplateId::from_raw("absent");
        let error = render(f.engine(), &absent, RenderContext::default()).await;
        assert_eq!(error.unwrap_err(), "not_found.template");
    })
    .await;
}
