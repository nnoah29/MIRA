#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Intégration au bureau du démon : notification d'attente du trousseau, absence de récepteur.

mod support;

use std::sync::Arc;

use mira_daemon::desktop::Desktop;
use mira_host::notifier::Texts;
use support::notifier_sink::CollectingSink;

/// L'attente du trousseau donne une notification muette, sans message à ouvrir, dans la langue du système.
#[test]
fn the_keyring_wait_is_announced_once_with_the_locale_texts() {
    let sink = Arc::new(CollectingSink::default());
    let desktop = Desktop::new(Some(sink.clone()), Texts::for_locale("fr_FR.UTF-8"));
    desktop.announce_keyring_locked();
    let shown = sink.shown();
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].title, "MIRA attend le déverrouillage du trousseau");
    assert!(!shown[0].sound && shown[0].message.is_none());
}

/// Sans récepteur (pas de bus de session), annoncer ne fait rien.
#[test]
fn without_a_sink_announcing_does_nothing() {
    let desktop = Desktop::new(None, Texts::for_locale("en"));
    desktop.announce_keyring_locked();
}
