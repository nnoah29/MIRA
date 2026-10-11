#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Désabonnement (11.6), aperçu et conversion HTML vers texte.

use mira_core::mime::{html_to_text, parse_message, parse_unsubscribe_headers, preview_text};

/// Un lien https et une adresse mailto sont lus, le « un clic » exige l'en-tête Post
#[test]
fn https_and_mailto_with_one_click() {
    let raw = b"Subject: n\r\nList-Unsubscribe: <mailto:stop@example.org?subject=stop>, <https://example.org/u/1>\r\nList-Unsubscribe-Post: List-Unsubscribe=One-Click\r\n\r\ncorps\r\n";
    let unsubscribe = parse_message(raw).unsubscribe.unwrap();
    assert_eq!(
        unsubscribe.https_url.as_deref(),
        Some("https://example.org/u/1")
    );
    assert_eq!(
        unsubscribe.mailto.as_deref(),
        Some("mailto:stop@example.org?subject=stop")
    );
    assert!(unsubscribe.one_click);
}

/// Sans List-Unsubscribe-Post, il n'y a pas de désabonnement en un clic
#[test]
fn without_post_header_it_is_not_one_click() {
    let raw = b"Subject: n\r\nList-Unsubscribe: <https://example.org/u/2>\r\n\r\ncorps\r\n";
    let unsubscribe = parse_message(raw).unsubscribe.unwrap();
    assert!(!unsubscribe.one_click && unsubscribe.mailto.is_none());
    assert!(
        parse_message(b"Subject: n\r\n\r\ncorps\r\n")
            .unsubscribe
            .is_none()
    );
}

/// Les en-têtes seuls suffisent pour lire le désabonnement
#[test]
fn headers_alone_expose_the_unsubscribe() {
    let headers = b"List-Unsubscribe: <mailto:bye@example.org>\r\n\r\n";
    let unsubscribe = parse_unsubscribe_headers(headers).unwrap();
    assert_eq!(
        unsubscribe.mailto.as_deref(),
        Some("mailto:bye@example.org")
    );
}

/// L'aperçu normalise les espaces et coupe sur une frontière de caractère
#[test]
fn preview_is_normalized_and_char_safe() {
    assert_eq!(
        preview_text("  Bonjour \r\n\r\n  le   monde  "),
        "Bonjour le monde"
    );
    let long = "é".repeat(500);
    let preview = preview_text(&long);
    assert_eq!(preview.chars().count(), 200);
    assert!(preview.chars().all(|c| c == 'é'));
}

/// La conversion HTML retire balises, scripts et styles et décode les entités
#[test]
fn html_is_reduced_to_text() {
    let html = "<div>Salut&nbsp;<b>toi</b></div><script>alert(1)</script><p>a &lt; b &#233;</p>";
    let text = html_to_text(html);
    assert!(text.contains("Salut toi"), "{text:?}");
    assert!(text.contains("a < b é"), "{text:?}");
    assert!(!text.contains("alert"), "{text:?}");
}
