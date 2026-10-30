#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Détecteurs de codes à usage unique, d'e-mails et de téléphones, catégories actives et chevauchements.

mod support_masking;

use mira_core::ai::masking::detect;
use mira_vocabulary::SensitiveCategory::{Card, Email, Iban, Otp, Phone};
use support_masking::{ALL, values};

/// Les chiffres près d'un mot-clé sont trouvés, des chiffres seuls ne le sont pas
#[test]
fn otp_near_keyword_is_found_and_bare_digits_are_not() {
    assert_eq!(
        values("Votre code de vérification est : 482913.", &[Otp]),
        ["482913"]
    );
    assert_eq!(values("Security code\n7391", &[Otp]), ["7391"]);
    assert!(values("J'ai 123456 pommes", &[Otp]).is_empty());
    assert!(values("code 123456789012", &[Otp]).is_empty());
    let far = format!("code{}1234", " ".repeat(41));
    assert!(values(&far, &[Otp]).is_empty());
}

/// Une adresse e-mail est trouvée quand sa catégorie est active, ponctuation autour exclue
#[test]
fn email_found_when_category_enabled() {
    let text = "Écrire à <marie.durand+tri@example.org>, ou a@b.fr.";
    assert_eq!(
        values(text, &[Email]),
        ["marie.durand+tri@example.org", "a@b.fr"]
    );
    assert!(values("pas d'adresse @ ici", &[Email]).is_empty());
}

/// Un téléphone de 8 chiffres ou plus est trouvé, pas une date ni un numéro de version
#[test]
fn phone_found_when_category_enabled() {
    let text = "Appelez le +33 6 12 34 56 78 ou le 01.23.45.67.89.";
    assert_eq!(
        values(text, &[Phone]),
        ["+33 6 12 34 56 78", "01.23.45.67.89"]
    );
    assert!(values("le 2024-01-15 à 10h, v1.2.3.4.5.6.7.8", &[Phone]).is_empty());
    assert!(values("tel 1234567", &[Phone]).is_empty());
}

/// Une catégorie absente de la liste n'est jamais cherchée
#[test]
fn disabled_category_is_ignored() {
    let text = "FR7630006000011234567890189 et a@b.fr et 4111111111111111";
    assert_eq!(values(text, &[Email]), ["a@b.fr"]);
    assert!(detect(text, &[]).is_empty());
    assert_eq!(detect(text, &[Iban, Card]).len(), 2);
}

/// Des valeurs collées ne se chevauchent pas, et la catégorie prioritaire l'emporte
#[test]
fn adjacent_findings_do_not_overlap() {
    let text = "4111111111111111 a@b.fr FR7630006000011234567890189";
    let found = detect(text, &ALL);
    assert_eq!(found.len(), 3);
    assert!(found.windows(2).all(|pair| pair[0].end <= pair[1].start));
    let digits = "4111 1111 1111 1111";
    let only = detect(digits, &ALL);
    assert_eq!((only.len(), only[0].category), (1, Card));
}

/// Du texte multioctet, y compris coupé près des valeurs, ne fait jamais paniquer
#[test]
fn detection_never_panics_on_multibyte_text() {
    let texts = [
        "é€漢字 FR7630006000011234567890189 ✓ ça",
        "🙂🙂 4111 1111 1111 1111 🙂 mot de passe : é漢🙂",
        "ＦＲ７６ ３０００ code ٤٥٦٧٨٩ ὈΔΥΣΣΕΎΣ a@é.fr +３３ ６ １２",
        "code é漢字🙂 123456 İstanbul ſk-aaaaaaaaaaaaaaaaaaaa",
    ];
    for text in texts {
        for cut in (0..=text.len()).filter(|index| text.is_char_boundary(*index)) {
            let found = detect(&text[..cut], &ALL);
            assert!(found.iter().all(|f| text.is_char_boundary(f.end)));
        }
    }
}
