#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Nettoyage du HTML d'origine : mutations XSS par confusion d'espace de noms, `<style>` étant admis.

use mira_core::html::sanitize_original_html;

/// Charges classiques de mutation (DOMPurify, ammonia) où un `<style>` change de nature au réencodage
const MUTATIONS: [&str; 5] = [
    "<svg><style><img src=x onerror=alert(1)></style></svg>",
    "<math><mtext><table><mglyph><style><img src=x onerror=alert(1)>",
    "<noscript><style></noscript><img src=x onerror=alert(1)></style>",
    r#"<svg></p><style><a title="</style><img src onerror=alert(1)>">"#,
    "<form><math><mtext></form><form><mglyph><style></math><img src onerror=alert(1)>",
];

/// Relit le HTML nettoyé : aucune attache d'événement ne doit survivre, ni au premier passage ni au second
#[test]
fn mutation_payloads_leave_no_event_handler() {
    for payload in MUTATIONS {
        let once = sanitize_original_html(payload);
        let twice = sanitize_original_html(&once);
        for html in [&once, &twice] {
            assert!(!html.contains("onerror"), "{payload} → {html}");
            assert!(!html.contains("alert(1)"), "{payload} → {html}");
        }
    }
}

/// Le nettoyage est stable : un second passage ne change plus rien (pas de mutation au réencodage)
#[test]
fn sanitizing_is_idempotent_on_mutation_payloads() {
    for payload in MUTATIONS {
        let once = sanitize_original_html(payload);
        assert_eq!(sanitize_original_html(&once), once, "{payload}");
    }
}
