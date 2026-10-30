#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Corrections de la tâche 3 : code postal et ponctuation de fin d'une valeur masquée.

mod support_masking;

use mira_vocabulary::SensitiveCategory::{Otp, Secret};
use support_masking::values;

/// Un code postal n'est pas un code à usage unique, un vrai code l'est toujours
#[test]
fn postal_code_is_not_an_otp() {
    for text in [
        "Code postal 75008",
        "code postale : 75008",
        "Zip code 90210",
        "postal code 12345",
        "CODE ZIP: 90210",
    ] {
        assert!(values(text, &[Otp]).is_empty(), "{text}");
    }
    assert_eq!(values("Votre code est 482913", &[Otp]), ["482913"]);
    assert_eq!(values("Code de vérification : 4829", &[Otp]), ["4829"]);
}

/// La ponctuation qui suit la valeur reste hors de la valeur masquée
#[test]
fn trailing_punctuation_stays_outside_the_masked_value() {
    let cases = [
        ("(mot de passe : Zx9-kp2.)", "Zx9-kp2"),
        ("mdp = Zx9kp2, puis", "Zx9kp2"),
        ("password: \"Zx9kp2\".", "Zx9kp2"),
        ("[password=Zx9kp2];", "Zx9kp2"),
        ("pwd: Zx9kp2?!", "Zx9kp2"),
    ];
    for (text, expected) in cases {
        assert_eq!(values(text, &[Secret]), [expected], "{text}");
    }
    let key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
    assert_eq!(values(&format!("(token: {key}),"), &[Secret]), [key]);
}
