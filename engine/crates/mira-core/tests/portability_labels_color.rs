#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Couleur d'un libellé dans le fichier d'export : `#AARRGGBB`, `#RRGGBB` accepté à l'import.

use mira_core::portability::labels::{format_color, parse_color};

/// L'écriture est en minuscules, sur huit chiffres
#[test]
fn format_is_lowercase_eight_digits() {
    assert_eq!(format_color(0xFF1E_90FF), "#ff1e90ff");
    assert_eq!(format_color(0), "#00000000");
}

/// Six chiffres veulent dire opaque
#[test]
fn parse_accepts_rrggbb_as_opaque() {
    assert_eq!(parse_color("#1e90ff"), Some(0xFF1E_90FF));
}

/// Huit chiffres portent l'alpha, en majuscules comme en minuscules
#[test]
fn parse_accepts_aarrggbb_any_case() {
    assert_eq!(parse_color("#80FFAA00"), Some(0x80FF_AA00));
    assert_eq!(parse_color("#80ffaa00"), Some(0x80FF_AA00));
}

/// Écrire puis relire rend la couleur d'origine
#[test]
fn parse_round_trips_every_format() {
    for color in [0, 1, 0x80FF_AA00, 0xFFFF_FFFF, 0xFFE8_542B] {
        assert_eq!(parse_color(&format_color(color)), Some(color));
    }
}

/// Toute autre forme est refusée
#[test]
fn parse_refuses_other_forms() {
    for text in [
        "#FFF",
        "FFF000",
        "rouge",
        "#GGGGGG",
        "#1234567",
        "#123456789",
        "",
        "#",
        "#+12345",
    ] {
        assert_eq!(parse_color(text), None, "{text:?}");
    }
}
