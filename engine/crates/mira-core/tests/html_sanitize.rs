#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rendu HTML sûr : pas de script, ressources distantes bloquées par défaut (11.6, 15.4).

use mira_core::html::sanitize_html;

/// Les scripts, gestionnaires d'événements et liens `javascript:` disparaissent
#[test]
fn scripts_handlers_and_javascript_links_are_removed() {
    let input = r#"<p onclick="steal()">Bonjour</p><script>alert(1)</script><a href="javascript:evil()">x</a><iframe src="https://e.test"></iframe>"#;
    let safe = sanitize_html(input, false);
    assert!(safe.html.contains("Bonjour"));
    for forbidden in ["<script", "onclick", "javascript:", "<iframe"] {
        assert!(
            !safe.html.contains(forbidden),
            "{forbidden} dans {}",
            safe.html
        );
    }
}

/// Un pixel espion distant est retiré et listé
#[test]
fn remote_tracking_pixel_is_blocked_and_listed() {
    let input = r#"<p>Promo</p><img src="https://tracker.test/p.gif?u=42" width="1" height="1">"#;
    let safe = sanitize_html(input, false);
    assert!(!safe.html.contains("tracker.test"));
    assert_eq!(
        safe.blocked_resources,
        vec!["https://tracker.test/p.gif?u=42".to_string()]
    );
}

/// Les images incorporées (`cid:`) restent, les fonds distants sont bloqués
#[test]
fn inline_images_stay_and_remote_backgrounds_are_blocked() {
    let input = r#"<img src="cid:logo@x"><table background="http://e.test/bg.png"><tr><td>a</td></tr></table>"#;
    let safe = sanitize_html(input, false);
    assert!(safe.html.contains("cid:logo@x"));
    assert!(!safe.html.contains("e.test"));
    assert_eq!(safe.blocked_resources.len(), 1);
}

/// Quand l'utilisateur autorise le contenu distant, les images sont gardées
#[test]
fn remote_content_is_kept_when_allowed() {
    let safe = sanitize_html(r#"<img src="https://cdn.test/photo.jpg">"#, true);
    assert!(safe.html.contains("https://cdn.test/photo.jpg"));
    assert!(safe.blocked_resources.is_empty());
}

/// Les styles en ligne (qui peuvent charger des URL) sont retirés et les liens neutralisés
#[test]
fn inline_styles_are_removed_and_links_get_safe_rel() {
    let input = r#"<div style="background:url(https://t.test/x)">a</div><a href="https://site.test">lien</a>"#;
    let safe = sanitize_html(input, false);
    assert!(!safe.html.contains("t.test"));
    assert!(safe.html.contains("noopener"));
}
