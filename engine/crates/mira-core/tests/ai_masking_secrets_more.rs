#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Secrets structurés (JWT, clé PEM, préfixes), mot-clé seul et texte multioctet.

mod support_masking;

use mira_vocabulary::SensitiveCategory::Secret;
use support_masking::{ALL, values};

const JWT: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

/// Trois segments base64url dont le premier commence par `eyJ` forment un jeton, seul ou après Bearer
#[test]
fn jwt_is_found() {
    assert_eq!(values(&format!("jeton {JWT}."), &[Secret]), [JWT]);
    assert_eq!(
        values(&format!("Authorization: Bearer {JWT}"), &[Secret]),
        [JWT]
    );
    assert!(values("eyJ.abc.def et a.b.c", &[Secret]).is_empty());
}

/// Un bloc de clé privée PEM est masqué en entier, de la ligne BEGIN à la ligne END
#[test]
fn pem_private_key_block_is_found_whole() {
    let body = "MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC7\nVJTUt9Us8cKjMzEfYyjiWA4R4/M2bS1GB4t7NXp98C3SC6dVMvDuictGeurT";
    let pem = format!("-----BEGIN PRIVATE KEY-----\n{body}\n-----END PRIVATE KEY-----");
    let text = format!("avant\n{pem}\naprès");
    assert_eq!(values(&text, &[Secret]), [pem.as_str()]);
    let rsa = "-----BEGIN RSA PRIVATE KEY-----\nQUJD\n-----END RSA PRIVATE KEY-----";
    assert_eq!(values(rsa, &[Secret]), [rsa]);
    assert!(
        values(
            "-----BEGIN PUBLIC KEY-----\nQUJD\n-----END PUBLIC KEY-----",
            &[Secret]
        )
        .is_empty()
    );
}

/// Les préfixes de GitLab, Stripe et npm sont reconnus
#[test]
fn extra_prefixes_are_found() {
    let gitlab = format!("glpat-{}", "aB3dE5gH7jK9mN1pQ3sT");
    let live = format!("sk_live_{}", "a1B2c3D4e5F6g7H8");
    let restricted = format!("rk_live_{}", "a1B2c3D4e5F6g7H8");
    let npm = format!("npm_{}", "aB3dE5gH7jK9mN1pQ3sT5vX7zA2cF4hJ6kL8");
    for secret in [gitlab, live, restricted, npm] {
        let text = format!("la clé {secret}, merci");
        assert_eq!(values(&text, &[Secret]), [secret.as_str()]);
    }
}

/// Un mot-clé sans valeur, ou avec une valeur trop courte, ne masque rien
#[test]
fn keyword_alone_without_value_is_untouched() {
    for text in [
        "token",
        "Le token : à renouveler",
        "api_key=",
        "secret: court",
        "bearer",
    ] {
        assert!(values(text, &[Secret]).is_empty(), "{text}");
    }
}

/// Du texte multioctet, coupé partout, ne fait jamais paniquer les nouveaux détecteurs
#[test]
fn detection_never_panics_on_multibyte_text() {
    let key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
    let text = format!(
        "é clé : \"{key}\" 漢 {JWT} 🙂 -----BEGIN PRIVATE KEY-----\nQUJD\n-----END PRIVATE KEY----- jeton=é{key}"
    );
    for cut in (0..=text.len()).filter(|index| text.is_char_boundary(*index)) {
        for found in mira_core::ai::masking::detect(&text[..cut], &ALL) {
            assert!(text.is_char_boundary(found.start) && text.is_char_boundary(found.end));
        }
    }
}
