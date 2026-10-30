#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Séparateurs de groupes d'un IBAN ou d'une carte : espace insécable, point, saut de ligne.

mod support_masking;

use mira_vocabulary::SensitiveCategory::{Card, Iban};
use support_masking::values;

/// Un IBAN dont les groupes sont séparés par des espaces insécables est trouvé en entier
#[test]
fn iban_with_non_breaking_spaces_is_found() {
    let text = "iban FR76\u{a0}3000\u{a0}6000\u{a0}0112\u{a0}3456\u{a0}7890\u{a0}189 merci";
    assert_eq!(
        values(text, &[Iban]),
        ["FR76\u{a0}3000\u{a0}6000\u{a0}0112\u{a0}3456\u{a0}7890\u{a0}189"]
    );
    let narrow = "FR76\u{202f}3000\u{202f}6000\u{202f}0112\u{202f}3456\u{202f}7890\u{202f}189";
    assert_eq!(values(narrow, &[Iban]), [narrow]);
}

/// Un IBAN coupé par un saut de ligne ou écrit avec des points est trouvé en entier
#[test]
fn iban_broken_over_lines_or_dotted_is_found() {
    let broken = "FR76 3000 6000 0112\n3456 7890 189";
    assert_eq!(values(broken, &[Iban]), [broken]);
    let windows = "FR76 3000 6000 0112\r\n3456 7890 189";
    assert_eq!(values(windows, &[Iban]), [windows]);
    let dotted = "FR76.3000.6000.0112.3456.7890.189";
    assert_eq!(values(dotted, &[Iban]), [dotted]);
}

/// Une carte dont les groupes sont séparés par des points ou des espaces insécables est trouvée
#[test]
fn card_with_dots_and_non_breaking_spaces_is_found() {
    assert_eq!(
        values("4111.1111.1111.1111", &[Card]),
        ["4111.1111.1111.1111"]
    );
    let text = "4111\u{a0}1111\u{a0}1111\u{a0}1111 et 4111\u{202f}1111\u{202f}1111\u{202f}1111";
    assert_eq!(
        values(text, &[Card]),
        [
            "4111\u{a0}1111\u{a0}1111\u{a0}1111",
            "4111\u{202f}1111\u{202f}1111\u{202f}1111"
        ]
    );
    assert!(values("4111 1111 1111  1111", &[Card]).is_empty());
}
