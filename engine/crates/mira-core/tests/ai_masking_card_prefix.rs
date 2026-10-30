#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Carte suivie d'un groupe de chiffres : le numéro valide est trouvé, le groupe en trop reste dehors.

mod support_masking;

use mira_vocabulary::SensitiveCategory::Card;
use support_masking::values;

/// Une carte suivie du cryptogramme est masquée seule
#[test]
fn card_followed_by_a_cvv_is_found_alone() {
    let text = "carte 4111 1111 1111 1111 123 merci";
    assert_eq!(values(text, &[Card]), ["4111 1111 1111 1111"]);
    assert_eq!(
        values("4111-1111-1111-1111-123", &[Card]),
        ["4111-1111-1111-1111"]
    );
}

/// Une carte suivie de sa date d'expiration est masquée seule
#[test]
fn card_followed_by_an_expiry_is_found_alone() {
    let text = "Card: 4111 1111 1111 1111 12/27";
    assert_eq!(values(text, &[Card]), ["4111 1111 1111 1111"]);
}

/// Un numéro de commande sans préfixe valide, une suite fausse et des chiffres dans un jeton ne sont pas trouvés
#[test]
fn numbers_without_a_valid_prefix_stay_unmasked() {
    assert!(values("commande 1234 5678 9012 3456 7890 12", &[Card]).is_empty());
    assert!(values("réf 4111 1111 1111 1112 123", &[Card]).is_empty());
    assert!(values("jeton a41111111111111111z", &[Card]).is_empty());
    assert!(values("41111111111111111234", &[Card]).is_empty());
}
