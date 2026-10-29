#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Conversion des dates RFC 3339 en secondes Unix.

use mira_core::ai::wire::rfc3339::unix_seconds;

/// L'époque, une année bissextile, un décalage négatif et une date antérieure à 1970
#[test]
fn unix_seconds_known_dates() {
    assert_eq!(unix_seconds("1970-01-01T00:00:00Z"), Some(0));
    assert_eq!(unix_seconds("2024-02-29T12:00:00Z"), Some(1_709_208_000));
    assert_eq!(
        unix_seconds("2024-02-29T07:00:00-05:00"),
        Some(1_709_208_000)
    );
    assert_eq!(unix_seconds("1969-12-31T23:59:59Z"), Some(-1));
}

/// La fraction de seconde est ignorée ; la lettre T et le Z minuscules sont admis
#[test]
fn unix_seconds_accepts_fraction_and_lowercase() {
    assert_eq!(
        unix_seconds("2000-01-01T00:00:00.123456Z"),
        Some(946_684_800)
    );
    assert_eq!(unix_seconds("2000-01-01t00:00:00z"), Some(946_684_800));
}

/// Les textes mal formés ou hors bornes donnent None
#[test]
fn unix_seconds_rejects_malformed_text() {
    for text in [
        "",
        "hier",
        "2025-02-19",
        "2025-02-19T00:00:00",
        "2025-13-01T00:00:00Z",
        "2025-02-32T00:00:00Z",
        "2025-02-19T24:00:00Z",
        "2025-02-19T00:61:00Z",
        "2025-02-19T00:00Z",
        "25-02-19T00:00:00Z",
        "2025-02-19T00:00:00+0100",
    ] {
        assert_eq!(unix_seconds(text), None, "{text}");
    }
}
