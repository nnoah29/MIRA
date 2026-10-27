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

use mira_vocabulary::{Rule, RuleAction};
use support_imap::within;
use support_rule_api::calls::create;
use support_rule_api::portability::export_yaml;
use support_rule_api::samples::{only, subject_rule};
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

/// Un libellé supprimé sous une règle fait échouer l'export avec une erreur typée
#[tokio::test]
async fn export_with_dangling_reference_fails_typed() {
    within(async {
        let f = Fixture::start().await;
        let (rule, label_id, _) = referencing_rule(&f).await;
        create(f.engine(), rule, "k-r").await.unwrap();
        let id = mira_vocabulary::LocalLabelId::from_raw(label_id);
        f.db.write(|tx| tx.delete_local_label(&id)).unwrap();
        let error = export_yaml(f.engine()).await.unwrap_err();
        assert_eq!(error, "invalid_input.parameter");
    })
    .await;
}
