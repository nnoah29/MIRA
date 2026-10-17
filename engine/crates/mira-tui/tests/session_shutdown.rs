//! Fin de session : un signal d'arrêt sort proprement, terminal rendu, avis écrit, puis moteur fermé.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use std::sync::Arc;

use mira_tui::app::TerminalGuard;
use mira_tui::app::run_loop::LoopHooks;
use mira_tui::app::session::run_session_parts;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use support::event_log::EventLog;
use support::notice_writer::NoticeWriter;
use support::recording_raw::RecordingRaw;
use support::recording_writer::RecordingWriter;
use support::world::TestWorld;
use tokio::sync::mpsc;

/// Garde de terminal dont les effets sont journalisés.
fn recorded_guard(log: &EventLog) -> TerminalGuard {
    let out = RecordingWriter { log: log.clone() };
    let raw = RecordingRaw { log: log.clone() };
    TerminalGuard::with_parts(Box::new(out), Box::new(raw)).unwrap()
}

/// Un signal injecté arrête la boucle ; le terminal est rendu, puis l'avis écrit moteur encore ouvert, puis le moteur fermé.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn injected_stop_signal_exits_cleanly_and_closes_engine() {
    let (engine, _keep) = TestWorld::new().split();
    let weak = Arc::downgrade(&engine);
    let log = EventLog::default();
    let guard = recorded_guard(&log);
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    let (keys, key_rx) = mpsc::unbounded_channel();
    let (stop, stop_rx) = mpsc::unbounded_channel();
    stop.send(()).unwrap();
    let hooks = LoopHooks {
        stop: Some(stop_rx),
        ..LoopHooks::default()
    };
    let mut notice = NoticeWriter {
        log: log.clone(),
        engine: weak.clone(),
    };
    let outcome = run_session_parts(engine, guard, &mut terminal, key_rx, hooks, &mut notice).await;
    assert!(outcome.is_ok(), "{outcome:?}");
    let expected = [
        "raw_on",
        "screen_enter",
        "screen_leave",
        "raw_off",
        "notice:Fermeture de MIRA…:moteur ouvert",
    ];
    assert_eq!(log.snapshot(), expected);
    assert!(weak.upgrade().is_none());
    drop(keys);
}
