//! Rafale de messages : la boucle en applique au plus un nombre borné avant de redessiner et de relire le clavier.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::VecDeque;

use mira_tui::app::burst::{BURST_LIMIT, Drained, drain_bounded};

/// 1 000 messages en attente : seuls `BURST_LIMIT` sont appliqués, le reste attend le tour suivant.
#[test]
fn drain_stops_after_burst_limit() {
    let mut queue: VecDeque<u32> = (0..1_000).collect();
    let mut applied = 0;
    let outcome = drain_bounded(
        &mut queue,
        |queue| queue.pop_front(),
        |_, _| {
            applied += 1;
            false
        },
        BURST_LIMIT,
    );
    assert_eq!(outcome, Drained::Budget);
    assert_eq!(applied, BURST_LIMIT);
    assert_eq!(BURST_LIMIT, 256);
    assert_eq!(queue.len(), 1_000 - 256);
}

/// Une file courte est vidée ; une demande de sortie arrête tout de suite.
#[test]
fn drain_empties_short_queue_and_stops_on_quit() {
    let mut queue: VecDeque<u32> = (0..3).collect();
    let outcome = drain_bounded(
        &mut queue,
        |queue| queue.pop_front(),
        |_, _| false,
        BURST_LIMIT,
    );
    assert_eq!(outcome, Drained::Empty);
    let mut queue: VecDeque<u32> = (0..10).collect();
    let outcome = drain_bounded(
        &mut queue,
        |queue| queue.pop_front(),
        |_, n| n == 2,
        BURST_LIMIT,
    );
    assert_eq!(outcome, Drained::Quit);
    assert_eq!(queue.len(), 7);
}
