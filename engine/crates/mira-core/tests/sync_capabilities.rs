#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dégradation par capacité (10.14) : même état final du miroir avec les quatre jeux de capacités.

mod support_sync;

use mira_testkit::imap::CapabilitySet;
use support_sync::mirror::mirror_rows;
use support_sync::scenarios::{changes_scenario, strategy_notes};
use support_sync::within;

/// Les quatre jeux de capacités du faux serveur
fn capability_sets() -> Vec<(&'static str, CapabilitySet)> {
    vec![
        ("complet", CapabilitySet::full()),
        ("sans QRESYNC", CapabilitySet::without_qresync()),
        ("sans MOVE", CapabilitySet::without_move()),
        ("minimal", CapabilitySet::minimal()),
    ]
}

/// Le miroir final est identique pour un serveur complet, sans QRESYNC, sans MOVE et minimal
#[tokio::test]
async fn same_final_mirror_for_every_capability_set() {
    within(async {
        let mut finals = Vec::new();
        for (name, caps) in capability_sets() {
            let world = changes_scenario(caps).await;
            finals.push((name, mirror_rows(&world.db)));
        }
        let expected = &finals[0].1;
        assert_eq!(expected.len(), 4);
        assert!(expected.iter().any(|r| r.subject == "Courrier 1" && r.seen));
        assert!(expected.iter().all(|r| r.subject != "Courrier 2"));
        for (name, rows) in &finals {
            assert_eq!(rows, expected, "jeu de capacités : {name}");
        }
    })
    .await;
}

/// Chaque repli choisi est journalisé comme événement de diagnostic (QRESYNC puis CONDSTORE absents)
#[tokio::test]
async fn chosen_fallback_is_logged_as_a_diagnostic() {
    within(async {
        let full = changes_scenario(CapabilitySet::full()).await;
        assert!(strategy_notes(&full).is_empty());
        let no_qresync = changes_scenario(CapabilitySet::without_qresync()).await;
        assert!(strategy_notes(&no_qresync)[0].contains("QRESYNC indisponible"));
        let minimal = changes_scenario(CapabilitySet::minimal()).await;
        assert!(strategy_notes(&minimal)[0].contains("CONDSTORE indisponible"));
    })
    .await;
}

/// La stratégie suit les capacités annoncées : QRESYNC, CONDSTORE (CHANGEDSINCE) ou comparaison complète
#[tokio::test]
async fn strategy_follows_the_announced_capabilities() {
    within(async {
        let log_of = |world: &support_sync::world::SyncWorld| world.store.command_log().join("\n");
        let full = log_of(&changes_scenario(CapabilitySet::full()).await);
        assert!(full.contains("QRESYNC") && !full.contains("CHANGEDSINCE"));
        let condstore = log_of(&changes_scenario(CapabilitySet::without_qresync()).await);
        assert!(condstore.contains("CHANGEDSINCE") && !condstore.contains("QRESYNC ("));
        let minimal = log_of(&changes_scenario(CapabilitySet::minimal()).await);
        assert!(!minimal.contains("CHANGEDSINCE") && !minimal.contains("CONDSTORE"));
    })
    .await;
}
