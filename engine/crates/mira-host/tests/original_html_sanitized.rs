#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Le HTML d'origine livré à la vue web est nettoyé côté moteur (audit SEC, constat 1c).

use mira_host::original_html::html_of_raw;

/// Un mail piégé perd script, redirection et formulaire, mais garde son texte et ses styles
#[test]
fn trapped_original_html_is_cleaned_before_delivery() {
    let raw = concat!(
        "From: a@example.test\r\nTo: b@example.test\r\nSubject: s\r\n",
        "Content-Type: text/html; charset=utf-8\r\n\r\n",
        r#"<meta http-equiv="refresh" content="0;url=https://e.test"><style>p{color:red}</style>"#,
        r#"<p onclick="x()">Bonjour</p><script>alert(1)</script><form action="https://e.test"></form>"#,
    );
    let html = html_of_raw(raw.as_bytes()).unwrap();
    assert!(html.contains("Bonjour") && html.contains("p{color:red}"));
    for forbidden in ["<script", "<meta", "<form", "onclick"] {
        assert!(!html.contains(forbidden), "{forbidden} dans {html}");
    }
}
