#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! État d'un flux : l'arrêt demandé l'emporte toujours sur un crédit disponible, et les crédits sont plafonnés.

use mira_daemon::session::{Stop, Stream};

/// Même avec des crédits disponibles, un flux arrêté ne prend jamais de crédit (aucun tirage au sort).
#[tokio::test]
async fn a_stopped_stream_never_takes_a_credit() {
    for _ in 0..300 {
        let stream = Stream::new(10);
        stream.stop(Stop::Cancelled);
        assert_eq!(stream.take_credit().await, Err(Stop::Cancelled));
    }
}

/// Des `credit` énormes répétés plafonnent la fenêtre au lieu de faire paniquer le sémaphore.
#[test]
fn credit_additions_are_capped_without_panicking() {
    let stream = Stream::new(u32::MAX);
    assert!(stream.available() <= Stream::MAX_CREDIT);
    for _ in 0..8 {
        stream.add_credit(u32::MAX);
    }
    assert_eq!(stream.available(), Stream::MAX_CREDIT);
}

/// La première raison d'arrêt est conservée.
#[test]
fn the_first_stop_reason_wins() {
    let stream = Stream::new(1);
    stream.stop(Stop::Lagged);
    stream.stop(Stop::Cancelled);
    assert_eq!(stream.reason(), Some(Stop::Lagged));
}
