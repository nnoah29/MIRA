#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Import YAML des libellés locaux : refus, doublon interne, couleur par défaut, plafond, rapport borné.

mod support;
mod support_api;
mod support_imap;
mod support_label_api;
mod support_mirror;
mod support_write;

use mira_vocabulary::{LabelImportOutcome as Outcome, LabelImportRefusal as Refusal};
use serde_json::json;
use support_imap::within;
use support_label_api::calls::{create, labels};
use support_label_api::files::{file_of, import_file, outcomes};
use support_label_api::seeds::refused;
use support_write::fixture::Fixture;

/// Deux noms qui ne diffèrent que par la casse : le second est refusé, un seul est écrit
#[tokio::test]
async fn duplicate_name_inside_file_refuses_second() {
    within(async {
        let f = Fixture::start().await;
        let yaml = file_of(vec![json!({"name": "compta"}), json!({"name": "Compta"})]);
        let report = import_file(f.engine(), &yaml, false, "k-imp")
            .await
            .unwrap();
        assert_eq!(
            outcomes(&report),
            vec![Outcome::Imported, refused(Refusal::DuplicateInFile)]
        );
        assert_eq!(labels(f.engine()).await.len(), 1);
    })
    .await;
}

/// Un nom vide ou trop long est refusé ; les libellés suivants sont importés
#[tokio::test]
async fn invalid_name_is_refused_and_others_imported() {
    within(async {
        let f = Fixture::start().await;
        let items = vec![
            json!({"name": ""}),
            json!({"name": "x".repeat(65)}),
            json!({"name": "Ok"}),
        ];
        let report = import_file(f.engine(), &file_of(items), false, "k-imp")
            .await
            .unwrap();
        let invalid = refused(Refusal::InvalidName);
        assert_eq!(
            outcomes(&report),
            vec![invalid.clone(), invalid, Outcome::Imported]
        );
    })
    .await;
}

/// Couleur illisible ou champ inconnu : seul ce libellé est refusé
#[tokio::test]
async fn invalid_color_and_unknown_field_are_refused() {
    within(async {
        let f = Fixture::start().await;
        let items = vec![
            json!({"name": "A", "color": "rouge"}),
            json!({"name": "B", "color": "#FFF"}),
            json!({"name": "C", "couleur": "#ff0000"}),
            json!({"name": "D"}),
        ];
        let report = import_file(f.engine(), &file_of(items), false, "k-imp")
            .await
            .unwrap();
        let color = refused(Refusal::InvalidColor);
        let expected = vec![
            color.clone(),
            color,
            refused(Refusal::InvalidLabel),
            Outcome::Imported,
        ];
        assert_eq!(outcomes(&report), expected);
    })
    .await;
}

/// Sans couleur dans le fichier, le libellé reçoit une couleur opaque de la palette
#[tokio::test]
async fn color_defaults_when_missing() {
    within(async {
        let f = Fixture::start().await;
        let yaml = file_of(vec![json!({"name": "Voyages"})]);
        import_file(f.engine(), &yaml, false, "k-imp")
            .await
            .unwrap();
        let color = labels(f.engine()).await[0].color;
        assert_eq!(color >> 24, 0xFF, "{color:#x}");
    })
    .await;
}

/// 190 libellés existants et 20 nouveaux : 10 importés, 10 refusés, 200 au total
#[tokio::test]
async fn cap_of_200_refuses_the_overflow() {
    within(async {
        let f = Fixture::start().await;
        for n in 0..190 {
            create(f.engine(), &format!("Ancien {n:03}"), &format!("k-{n}"))
                .await
                .unwrap();
        }
        let items = (0..20)
            .map(|n| json!({"name": format!("Nouveau {n:02}")}))
            .collect();
        let report = import_file(f.engine(), &file_of(items), false, "k-imp")
            .await
            .unwrap();
        let counted = |wanted: &Outcome| outcomes(&report).iter().filter(|o| *o == wanted).count();
        assert_eq!(counted(&Outcome::Imported), 10);
        assert_eq!(counted(&refused(Refusal::TooManyLabels)), 10);
        assert_eq!(labels(f.engine()).await.len(), 200);
    })
    .await;
}

/// Le nom d'un libellé refusé est tronqué dans le rapport
#[tokio::test]
async fn oversized_name_in_report_is_truncated() {
    within(async {
        let f = Fixture::start().await;
        let yaml = file_of(vec![json!({"name": "é".repeat(100_000)})]);
        let report = import_file(f.engine(), &yaml, false, "k-imp")
            .await
            .unwrap();
        assert_eq!(outcomes(&report), vec![refused(Refusal::InvalidName)]);
        assert!(report.entries[0].name.chars().count() <= 64);
    })
    .await;
}
