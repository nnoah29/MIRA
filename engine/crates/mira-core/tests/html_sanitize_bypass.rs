#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contournements du blocage du contenu distant (audit 2/19, revue 10) : liste blanche stricte.

use mira_core::html::sanitize_html;

/// Vérifie qu'aucune ressource distante ne reste chargeable depuis ce HTML assaini
fn assert_blocked(input: &str, forbidden: &str) {
    let safe = sanitize_html(input, false);
    assert!(
        !safe.html.contains(forbidden),
        "{forbidden} subsiste dans {}",
        safe.html
    );
}

/// Une tabulation, un saut de ligne ou un contrôle C0 dans l'URL ne l'exempte pas
#[test]
fn regression_control_characters_in_url_do_not_bypass_blocking() {
    assert_blocked(r#"<img src="ht&#9;tps://t.test/p.gif">"#, "t.test");
    assert_blocked(r#"<img src="ht&#10;tp://t.test/p.gif">"#, "t.test");
    assert_blocked(r#"<img src="&#1;https://t.test/p.gif">"#, "t.test");
    assert_blocked(r#"<img src="  https://t.test/p.gif">"#, "t.test");
    let listed = sanitize_html(r#"<img src="ht&#9;tps://t.test/p.gif">"#, false);
    assert_eq!(listed.blocked_resources.len(), 1);
}

/// Les formes à barre oblique inverse et les URL sans schéma sont des ressources distantes
#[test]
fn regression_backslash_and_scheme_relative_urls_are_blocked() {
    assert_blocked(r#"<img src="\\t.test\p.gif">"#, "t.test");
    assert_blocked(r#"<img src="/\t.test/p.gif">"#, "t.test");
    assert_blocked(r#"<img src="//t.test/p.gif">"#, "t.test");
    assert_blocked(r#"<img src="t.test/p.gif">"#, "t.test");
}

/// L'attribut `background` n'accepte que les ressources incorporées
#[test]
fn regression_background_attribute_blocks_anything_but_embedded_resources() {
    assert_blocked(
        r#"<table><tr><td background="&#9;//t.test/bg.png">a</td></tr></table>"#,
        "t.test",
    );
    let safe = sanitize_html(r#"<td background="cid:bg@x">a</td>"#, false);
    assert!(safe.blocked_resources.is_empty());
}

/// `data:` est interdit dans un lien, et seules les images hors SVG sont admises dans `img`
#[test]
fn regression_data_urls_only_allowed_for_non_svg_images() {
    let link = sanitize_html(r#"<a href="data:text/html,<b>x</b>">x</a>"#, false);
    assert!(!link.html.contains("data:"), "{}", link.html);
    let link = sanitize_html(r#"<a href="da&#9;ta:text/html,x">x</a>"#, true);
    assert!(!link.html.contains("data:"), "{}", link.html);
    assert_blocked(r#"<img src="data:image/svg+xml,<svg/>">"#, "svg");
    assert_blocked(r#"<img src="data:text/html,x">"#, "text/html");
    let png = sanitize_html(r#"<img src="data:image/png;base64,AAAA">"#, false);
    assert!(png.html.contains("data:image/png"), "{}", png.html);
}

/// Une URL relative est refusée, mais les liens http(s) et mailto restent
#[test]
fn regression_relative_urls_are_denied_and_absolute_links_kept() {
    let safe = sanitize_html(
        r#"<a href="/x">r</a><a href="https://ok.test/">h</a><a href="mailto:a@b.test">m</a>"#,
        false,
    );
    assert!(!safe.html.contains(r#"href="/x""#), "{}", safe.html);
    assert!(safe.html.contains("https://ok.test/"));
    assert!(safe.html.contains("mailto:a@b.test"));
}
