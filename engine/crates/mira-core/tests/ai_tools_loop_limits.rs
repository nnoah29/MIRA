#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Boucle d'outils, limites : tours, taille des résultats, outils inconnus, arguments invalides, flux coupé.

mod ai_loop_support;
mod ai_models_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_loop_support::fixture::LoopFixture;
use ai_loop_support::turns::{FORMATS, collect, shape, text_turn, tool_results, tool_turn};
use mira_testkit::ai::Behavior;
use mira_vocabulary::{ChatDelta, StopReason};

const SEARCH: &str = r#"{"query":"devis"}"#;

/// Le neuvième tour n'est jamais demandé : huit requêtes exactement, fin par plafond, huit-septième appel exécuté
#[tokio::test]
async fn ninth_round_is_never_requested() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        for _ in 0..9 {
            fixture.queue(tool_turn(
                format,
                "",
                &[("c1", "search_messages", SEARCH)],
                None,
            ));
        }
        let mut stream = fixture.ask("Boucle").await;
        let deltas = collect(&mut stream).await;
        assert_eq!(fixture.bodies().len(), 8, "{format:?}");
        let last = deltas.last().unwrap();
        assert_eq!(
            last,
            &ChatDelta::Done {
                reason: StopReason::MaxToolRounds
            }
        );
        let called = shape(&deltas)
            .iter()
            .filter(|s| s.starts_with("called:"))
            .count();
        assert_eq!(called, 7);
    }
}

/// Un message énorme est coupé : le résultat tient en 8 000 caractères et porte la marque
#[tokio::test]
async fn oversize_result_is_cut_and_marked() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        fixture.set_body("m2", &"z".repeat(20_000));
        fixture.queue(tool_turn(
            format,
            "",
            &[("c1", "get_message", r#"{"message":"m2"}"#)],
            None,
        ));
        fixture.queue(text_turn(format, "Long.", None));
        collect(&mut fixture.ask("Lis m2").await).await;
        let result = &tool_results(&fixture.bodies()[1], format)[0];
        assert!(result.chars().count() <= 8_100, "{format:?}");
        assert!(result.contains("…[tronqué]"));
    }
}

/// Un outil inconnu reçoit un résultat d'erreur et la boucle continue
#[tokio::test]
async fn unknown_tool_gets_error_result_and_loop_continues() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        fixture.queue(tool_turn(
            format,
            "",
            &[("c1", "delete_everything", "{}")],
            None,
        ));
        fixture.queue(text_turn(format, "Ok.", None));
        let deltas = collect(&mut fixture.ask("Efface tout").await).await;
        assert!(
            shape(&deltas).contains(&"finished:error".to_string()),
            "{format:?}"
        );
        let result = &tool_results(&fixture.bodies()[1], format)[0];
        assert!(result.contains("outil inconnu"));
        assert_eq!(
            deltas.last(),
            Some(&ChatDelta::Done {
                reason: StopReason::EndTurn
            })
        );
    }
}

/// Des arguments illisibles reçoivent un résultat d'erreur, pas une panique
#[tokio::test]
async fn invalid_arguments_get_error_result() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        fixture.queue(tool_turn(
            format,
            "",
            &[("c1", "search_messages", "{pas du json")],
            None,
        ));
        fixture.queue(text_turn(format, "Ok.", None));
        collect(&mut fixture.ask("Cherche").await).await;
        let result = &tool_results(&fixture.bodies()[1], format)[0];
        assert!(result.contains("arguments illisibles"), "{format:?}");
    }
}

/// Un flux coupé au milieu des arguments échoue proprement, sans exécuter l'appel
#[tokio::test]
async fn stream_cut_inside_arguments_ends_failed_without_panic() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        let Behavior::Stream { chunks, .. } =
            tool_turn(format, "", &[("c1", "search_messages", SEARCH)], None)
        else {
            panic!("flux attendu")
        };
        let cut = chunks[..chunks.len() - 3].to_vec();
        fixture.queue(Behavior::DropAfter { chunks: cut });
        let deltas = collect(&mut fixture.ask("Cherche").await).await;
        assert!(
            !shape(&deltas).iter().any(|s| s.starts_with("called:")),
            "{format:?}"
        );
        assert!(matches!(
            deltas.last(),
            Some(ChatDelta::Done {
                reason: StopReason::Failed { .. }
            })
        ));
        assert_eq!(fixture.bodies().len(), 1);
    }
}

/// Au-delà de huit appels dans un tour, les suivants reçoivent une erreur : chaque appel a son résultat
#[tokio::test]
async fn too_many_calls_in_one_turn_get_error_results() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        let ids: Vec<String> = (0..10).map(|n| format!("c{n}")).collect();
        let calls: Vec<(&str, &str, &str)> = ids
            .iter()
            .map(|id| (id.as_str(), "list_mailboxes", "{}"))
            .collect();
        fixture.queue(tool_turn(format, "", &calls, None));
        fixture.queue(text_turn(format, "Fini.", None));
        let deltas = collect(&mut fixture.ask("Beaucoup").await).await;
        let shape = shape(&deltas);
        assert_eq!(
            shape.iter().filter(|s| *s == "finished:ok").count(),
            8,
            "{format:?}"
        );
        assert_eq!(shape.iter().filter(|s| *s == "finished:error").count(), 2);
        assert_eq!(tool_results(&fixture.bodies()[1], format).len(), 10);
    }
}
