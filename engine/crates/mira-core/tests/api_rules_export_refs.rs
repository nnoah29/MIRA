#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Export YAML des règles : références en clair, cible disparue.

mod support;
mod support_api;
mod support_imap;
mod support_label_api;
mod support_mirror;
mod support_rule_api;
mod support_template_api;
mod support_write;

use mira_core::portability::envelope::unwrap;
use mira_vocabulary::{Rule, RuleAction};
use support_imap::within;
use support_rule_api::calls::create;
use support_rule_api::portability::{export_response, export_yaml};
use support_rule_api::samples::{only, subject_rule};
use support_rule_api::writes::create_named;
use support_write::fixture::Fixture;

/// Règle du compte qui classe, étiquette et répond avec un modèle
async fn referencing_rule(f: &Fixture) -> (Rule, String, String) {
    let label = support_label_api::calls::create(f.engine(), "Compta", "k-l")
        .await
        .unwrap();
    let template = support_template_api::calls::create(f.engine(), "Accusé", "Reçu", "k-t")
        .await
        .unwrap();
    let mut rule = subject_rule("Factures", "Facture", only(&f.account));
    rule.actions = vec![
        RuleAction::MoveTo {
            mailbox: mira_vocabulary::MailboxId::from_raw("box-in"),
        },
        RuleAction::AddLocalLabel {
            label: label.id.clone(),
        },
        RuleAction::ReplyWithTemplate {
            template: template.id.clone(),
        },
    ];
    (
        rule,
        label.id.as_str().to_string(),
        template.id.as_str().to_string(),
    )
}

/// Le fichier contient adresse, chemin de boîte et noms ; aucun identifiant local
#[tokio::test]
async fn export_uses_portable_references() {
    within(async {
        let f = Fixture::start().await;
        let (rule, label_id, template_id) = referencing_rule(&f).await;
        create(f.engine(), rule, "k-r").await.unwrap();
        let yaml = export_yaml(f.engine()).await.unwrap();
        for clear in [support_write::USER, "Compta", "Accusé", "INBOX"] {
            assert!(yaml.contains(clear), "{clear} absent de {yaml}");
        }
        for local in [f.account.as_str(), "box-in", &label_id, &template_id] {
            assert!(!yaml.contains(local), "{local} présent dans {yaml}");
        }
    })
    .await;
}

/// Une règle dont le libellé a disparu est laissée de côté et nommée ; les autres sont exportées
#[tokio::test]
async fn export_skips_rule_with_dangling_reference_and_names_it() {
    within(async {
        let f = Fixture::start().await;
        let (rule, label_id, _) = referencing_rule(&f).await;
        create(f.engine(), rule, "k-r").await.unwrap();
        create_named(f.engine(), &["Saine"]).await;
        let id = mira_vocabulary::LocalLabelId::from_raw(label_id);
        f.db.write(|tx| tx.delete_local_label(&id)).unwrap();
        let (yaml, skipped) = export_response(f.engine()).await.unwrap();
        let items = unwrap(&yaml, "rules", "rules").unwrap();
        let names: Vec<_> = items
            .iter()
            .map(|item| item["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["Saine"]);
        assert_eq!(skipped, ["Factures"]);
    })
    .await;
}
