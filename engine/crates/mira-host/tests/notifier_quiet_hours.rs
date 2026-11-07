#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Heures calmes : plage du même jour, plage passant minuit, absence de plage, effet sur la décision.

use mira_assembly::mira_vocabulary::{AccountId, EngineSettings, QuietHours};
use mira_host::notifier::{Decision, decide, in_quiet_hours};

/// Heures calmes de `from` à `to`.
fn quiet(from: u8, to: u8) -> Option<QuietHours> {
    Some(QuietHours { from, to })
}

/// Une plage du même jour inclut son début et exclut sa fin.
#[test]
fn a_same_day_range_includes_its_start_and_excludes_its_end() {
    assert!(in_quiet_hours(quiet(9, 17), 9));
    assert!(in_quiet_hours(quiet(9, 17), 16));
    assert!(!in_quiet_hours(quiet(9, 17), 17));
    assert!(!in_quiet_hours(quiet(9, 17), 8));
}

/// Une plage de 22 h à 7 h couvre la nuit, minuit compris.
#[test]
fn a_range_across_midnight_covers_the_night() {
    for hour in [22, 23, 0, 3, 6] {
        assert!(in_quiet_hours(quiet(22, 7), hour), "{hour}");
    }
    for hour in [7, 12, 21] {
        assert!(!in_quiet_hours(quiet(22, 7), hour), "{hour}");
    }
}

/// Sans plage, ou avec un début égal à la fin, jamais calme.
#[test]
fn no_range_or_an_empty_range_is_never_quiet() {
    assert!(!in_quiet_hours(None, 3));
    assert!(!in_quiet_hours(quiet(5, 5), 5));
}

/// Pendant les heures calmes, la décision est de ne pas notifier.
#[test]
fn quiet_hours_make_the_decision_skip() {
    let settings = EngineSettings {
        quiet_hours: quiet(22, 7),
        ..EngineSettings::default()
    };
    let account = AccountId::from_raw("a1");
    assert_eq!(
        decide(&settings, false, &account, false, 23),
        Decision::Skip
    );
    assert_eq!(
        decide(&settings, false, &account, false, 12),
        Decision::Show
    );
}
