#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Détecteurs d'IBAN, de cartes et de secrets : formes valides, contrôles et mots-clés.

mod support_masking;

use mira_vocabulary::SensitiveCategory::{Card, Iban, Secret};
use support_masking::values;

/// Un IBAN au contrôle valide est trouvé, un contrôle faux ne l'est pas, ponctuation et lignes autour comprises
#[test]
fn valid_iban_is_found_and_invalid_checksum_is_not() {
    let valid = "Mon IBAN : FR7630006000011234567890189.\nMerci !";
    assert_eq!(values(valid, &[Iban]), ["FR7630006000011234567890189"]);
    let wrong = "Mon IBAN : FR7630006000011234567890188.";
    assert!(values(wrong, &[Iban]).is_empty());
    let german = "(DE89370400440532013000), merci";
    assert_eq!(values(german, &[Iban]), ["DE89370400440532013000"]);
}

/// Espaces, tirets et minuscules sont tolérés, et le mot qui suit n'est pas avalé
#[test]
fn iban_with_spaces_and_lowercase_separators() {
    let spaced = "iban FR76 3000 6000 0112 3456 7890 189 merci";
    assert_eq!(
        values(spaced, &[Iban]),
        ["FR76 3000 6000 0112 3456 7890 189"]
    );
    let lower = "gb82 west 1234 5698 7654 32 et fr76-3000-6000-0112-3456-7890-189";
    assert_eq!(
        values(lower, &[Iban]),
        [
            "gb82 west 1234 5698 7654 32",
            "fr76-3000-6000-0112-3456-7890-189"
        ]
    );
}

/// Un numéro de carte au contrôle de Luhn est trouvé, une suite de chiffres quelconque ne l'est pas
#[test]
fn card_with_luhn_is_found_and_random_digits_are_not() {
    assert_eq!(
        values("Carte 4111111111111111 !", &[Card]),
        ["4111111111111111"]
    );
    assert_eq!(values("(378282246310005)", &[Card]), ["378282246310005"]);
    assert!(values("Réf 4111 1111 1111 1112", &[Card]).is_empty());
    assert!(values("Commande 1234567890123", &[Card]).is_empty());
}

/// Les groupes séparés par un espace ou un tiret sont reconnus
#[test]
fn card_with_spaces_and_hyphens() {
    let text = "4111 1111 1111 1111 puis 5500-0000-0000-0004.";
    assert_eq!(
        values(text, &[Card]),
        ["4111 1111 1111 1111", "5500-0000-0000-0004"]
    );
}

/// Les préfixes de clés connus sont trouvés ; un mot qui ressemble sans l'être ne l'est pas
#[test]
fn known_api_key_prefixes_are_found() {
    let keys = [
        "sk-proj1234567890abcdefgh",
        "sk-ant-api03-abcdefghijklmnop",
        "ghp_abcdefghijklmnopqrstuvwxyz0123456789",
        "AKIAIOSFODNN7EXAMPLE",
        "xoxb-1234567890-abcdefghij",
        "AIzaSyA1234567890abcdefghijklmnopqrstu",
    ];
    for key in keys {
        let text = format!("clé : {key}, fin");
        assert_eq!(values(&text, &[Secret]), [key]);
    }
    assert!(values("sk-court et task-abcdefghijklmnopqrstuvwxyz", &[Secret]).is_empty());
}

/// Après un mot-clé suivi de `:` ou `=`, seule la valeur est masquée
#[test]
fn password_value_after_keyword_is_masked_not_the_keyword() {
    assert_eq!(
        values("Mot de passe : Tr0ub4dor&3 merci", &[Secret]),
        ["Tr0ub4dor&3"]
    );
    assert_eq!(values("PASSWORD=hunter2\nsuite", &[Secret]), ["hunter2"]);
    assert_eq!(values("mdp: abc\nfin", &[Secret]), ["abc"]);
    assert_eq!(values("pwd =\nsuite", &[Secret]), Vec::<&str>::new());
    assert!(values("le mot de passe est oublié", &[Secret]).is_empty());
}
