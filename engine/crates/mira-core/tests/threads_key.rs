#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Clé de fil (D-15) : ordre de priorité des en-têtes et normalisation de l'objet.

mod support;
mod support_mirror;
mod support_reading;

use mira_core::threads::{normalize_subject, strip_reply_prefixes, thread_key};
use support_reading::threading::threaded;

/// Le premier élément de `References` l'emporte sur tout le reste
#[test]
fn key_is_first_reference() {
    let message = threaded(
        "m",
        "S",
        Some("<own>"),
        Some("<parent>"),
        &["<root>", "<mid>"],
    );
    assert_eq!(thread_key(&message, Some("srv")), "<root>");
}

/// Sans `References`, `In-Reply-To` désigne la racine
#[test]
fn key_falls_back_to_in_reply_to() {
    let message = threaded("m", "S", Some("<own>"), Some("<parent>"), &[]);
    assert_eq!(thread_key(&message, None), "<parent>");
}

/// L'indice serveur ne compte que sans en-tête de réponse, et prime alors sur le `Message-ID`
#[test]
fn key_uses_hint_only_without_reply_headers() {
    let plain = threaded("m", "S", Some("<own>"), None, &[]);
    assert_eq!(thread_key(&plain, Some("t42")), "hint:t42");
    let reply = threaded("r", "S", Some("<own>"), Some("<parent>"), &[]);
    assert_eq!(thread_key(&reply, Some("t42")), "<parent>");
}

/// Sans aucun en-tête de fil, on retombe sur le `Message-ID` puis sur l'objet normalisé
#[test]
fn key_falls_back_to_own_id_then_subject() {
    let with_id = threaded("m", "Re: Bonjour", Some(" <own> "), None, &[]);
    assert_eq!(thread_key(&with_id, None), "<own>");
    let bare = threaded(
        "n",
        "RE: Fwd:  Bonjour  Monde ",
        None,
        Some("  "),
        &["", " "],
    );
    assert_eq!(thread_key(&bare, None), "subject:bonjour monde");
}

/// Les préfixes de réponse et de transfert sont retirés de façon répétée, casse et espaces indifférents
#[test]
fn subject_prefixes_are_stripped_repeatedly() {
    for subject in [
        "Re: Plan",
        "RE: re: Plan",
        "Fwd: Fw: Plan",
        "TR: Plan",
        "AW: WG: Plan",
        "Réf : Plan",
        "  réf: RE :  Plan  ",
    ] {
        assert_eq!(normalize_subject(subject), "plan", "objet {subject:?}");
    }
}

/// Un objet qui ressemble à un préfixe sans l'être reste intact, et la casse d'origine est gardée
#[test]
fn subject_lookalikes_are_kept() {
    assert_eq!(normalize_subject("Reunion demain"), "reunion demain");
    assert_eq!(
        normalize_subject("Référence: dossier"),
        "référence: dossier"
    );
    assert_eq!(strip_reply_prefixes("Re: Fwd: Plan B"), "Plan B");
    assert_eq!(strip_reply_prefixes("Re:"), "");
}
