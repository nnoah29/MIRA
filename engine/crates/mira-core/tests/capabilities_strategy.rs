#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Choix de stratégie selon quatre profils de serveur : complet, sans QRESYNC, sans MOVE, minimal.

use mira_core::capabilities::{
    ChangeDetection, EffectiveCapabilities, ExpungeMethod, MoveMethod, Realtime,
    choose_change_detection, choose_expunge, choose_move, choose_realtime, parse_capabilities,
};
use mira_vocabulary::ProviderQuirks;

/// Construit les capacités effectives d'un serveur qui annonce ces jetons
fn server(announced: &[&str]) -> EffectiveCapabilities {
    let tokens: Vec<String> = announced.iter().map(ToString::to_string).collect();
    EffectiveCapabilities::compute(&parse_capabilities(&tokens), &ProviderQuirks::default())
}

/// Profil complet : les stratégies les plus fines
#[test]
fn full_profile_picks_the_finest_strategies() {
    let capabilities = server(&["IDLE", "CONDSTORE", "QRESYNC", "MOVE", "UIDPLUS", "ENABLE"]);
    assert_eq!(
        choose_change_detection(&capabilities),
        ChangeDetection::Qresync
    );
    assert_eq!(choose_move(&capabilities), MoveMethod::Move);
    assert_eq!(choose_realtime(&capabilities), Realtime::Idle);
    assert_eq!(choose_expunge(&capabilities), ExpungeMethod::UidExpunge);
}

/// Sans QRESYNC : CONDSTORE prend le relais, le reste ne change pas
#[test]
fn without_qresync_condstore_takes_over() {
    let capabilities = server(&["IDLE", "CONDSTORE", "MOVE", "UIDPLUS"]);
    assert_eq!(
        choose_change_detection(&capabilities),
        ChangeDetection::Condstore
    );
    assert_eq!(choose_move(&capabilities), MoveMethod::Move);
}

/// Sans MOVE : copie puis suppression
#[test]
fn without_move_copy_then_delete() {
    let capabilities = server(&["IDLE", "CONDSTORE", "QRESYNC", "UIDPLUS"]);
    assert_eq!(choose_move(&capabilities), MoveMethod::CopyThenDelete);
    assert_eq!(
        choose_change_detection(&capabilities),
        ChangeDetection::Qresync
    );
}

/// Profil minimal : comparaison complète, copie puis suppression, interrogation, aucune purge
#[test]
fn minimal_profile_falls_back_everywhere() {
    let capabilities = server(&["IMAP4rev1"]);
    assert_eq!(
        choose_change_detection(&capabilities),
        ChangeDetection::FullCompare
    );
    assert_eq!(choose_move(&capabilities), MoveMethod::CopyThenDelete);
    assert_eq!(choose_realtime(&capabilities), Realtime::Polling);
    assert_eq!(choose_expunge(&capabilities), ExpungeMethod::NoPurge);
}

/// Une exception du provider change la stratégie choisie
#[test]
fn provider_exception_changes_the_strategy() {
    let announced = parse_capabilities(&["QRESYNC".to_string(), "CONDSTORE".to_string()]);
    let quirks = ProviderQuirks {
        unreliable_extensions: vec!["QRESYNC".to_string()],
        ..ProviderQuirks::default()
    };
    let capabilities = EffectiveCapabilities::compute(&announced, &quirks);
    assert_eq!(
        choose_change_detection(&capabilities),
        ChangeDetection::Condstore
    );
}
