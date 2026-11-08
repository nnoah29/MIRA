//! Le corps d'une notification est borné quel que soit l'objet du mail.

use mira_daemon::desktop::bound_body;

/// Un long objet est coupé à 400 caractères, un court reste intact.
#[test]
fn a_long_body_is_cut_at_400_characters_and_a_short_one_is_kept() {
    assert_eq!(bound_body(&"é".repeat(1000)).chars().count(), 400);
    assert_eq!(bound_body("Bonjour"), "Bonjour");
}
