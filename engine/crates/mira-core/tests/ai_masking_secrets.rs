#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Secrets génériques : mot-clé puis valeur, seuils d'entropie, UUID et absence de mot-clé.

mod support_masking;

use mira_core::ai::masking::shannon;
use mira_vocabulary::SensitiveCategory::Secret;
use support_masking::values;

const AWS_EXAMPLE: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
const EMPTY_SHA1: &str = "da39a3ee5e6b4b0d3255bfef95601890afd80709";

/// Une chaîne constante vaut 0 bit, deux symboles équiprobables 1 bit, quatre 2 bits
#[test]
fn shannon_entropy_of_known_strings() {
    assert!(shannon("aaaaaaaa").abs() < 1e-9);
    assert!((shannon("abababab") - 1.0).abs() < 1e-9);
    assert!((shannon("abcdabcd") - 2.0).abs() < 1e-9);
    assert!(shannon("").abs() < 1e-9);
    assert!(shannon("é漢é漢") > 0.99);
}

/// Une valeur riche en entropie après un mot-clé est trouvée, la valeur seule
#[test]
fn generic_secret_after_keyword_with_high_entropy_is_found() {
    let text = format!("aws_secret_access_key = {AWS_EXAMPLE}\nok");
    assert_eq!(values(&text, &[Secret]), [AWS_EXAMPLE]);
    let json = format!(r#"{{"api_key": "{AWS_EXAMPLE}"}}"#);
    assert_eq!(values(&json, &[Secret]), [AWS_EXAMPLE]);
    let header = format!("Authorization: Bearer {AWS_EXAMPLE}.");
    assert_eq!(values(&header, &[Secret]), [AWS_EXAMPLE]);
    let french = format!("Le jeton : {AWS_EXAMPLE}");
    assert_eq!(values(&french, &[Secret]), [AWS_EXAMPLE]);
}

/// Une valeur longue mais répétitive après un mot-clé n'est pas un secret
#[test]
fn low_entropy_value_after_keyword_is_not_found() {
    assert!(values("token = aaaaaaaaaaaaaaaaaaaaaaaaaaaa", &[Secret]).is_empty());
    assert!(values("secret: changemechangemechangeme", &[Secret]).is_empty());
    assert!(values("api_key=abababababababababababab", &[Secret]).is_empty());
}

/// Un UUID après un mot-clé n'est jamais masqué
#[test]
fn uuid_after_keyword_is_not_masked() {
    let uuid = "123e4567-e89b-12d3-a456-426614174000";
    assert!(values(&format!("token: {uuid}"), &[Secret]).is_empty());
    assert!(values(&format!("api_key={}", uuid.to_uppercase()), &[Secret]).is_empty());
}

/// Une valeur hexadécimale d'au moins 32 caractères passe au seuil 3,0, pas une plus courte
#[test]
fn long_hex_after_keyword_uses_the_lower_threshold() {
    assert!(shannon(EMPTY_SHA1) < 4.5);
    assert_eq!(
        values(&format!("access_key = {EMPTY_SHA1}"), &[Secret]),
        [EMPTY_SHA1]
    );
    let short = "da39a3ee5e6b4b0d3255bfef95";
    assert!(values(&format!("token={short}"), &[Secret]).is_empty());
    let flat = "0".repeat(31) + "1";
    assert!(values(&format!("token={flat}"), &[Secret]).is_empty());
}

/// Sans mot-clé, une valeur riche en entropie n'est jamais détectée
#[test]
fn high_entropy_without_keyword_is_not_found() {
    assert!(values(&format!("voici {AWS_EXAMPLE} comme ça"), &[Secret]).is_empty());
    assert!(values(&format!("hash: {EMPTY_SHA1}"), &[Secret]).is_empty());
}
