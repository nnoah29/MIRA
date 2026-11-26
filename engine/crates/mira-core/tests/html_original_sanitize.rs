#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Nettoyage léger du HTML d'origine : fidélité du rendu, aucun élément actif (audit SEC, constat 1c).

use mira_core::html::sanitize_original_html;

/// Les éléments actifs et ceux qui redirigent ou changent la base disparaissent
#[test]
fn active_and_navigating_elements_are_removed() {
    let input = concat!(
        r#"<meta http-equiv="refresh" content="0;url=https://e.test"><base href="https://e.test/">"#,
        r#"<p onclick="x()" onmouseover="y()">Bonjour</p><script>alert(1)</script>"#,
        r#"<iframe src="https://e.test"></iframe><object data="a"></object><embed src="a">"#,
        r#"<form action="https://e.test"><input name="a"></form><a href="javascript:evil()">l</a>"#,
    );
    let html = sanitize_original_html(input);
    assert!(html.contains("Bonjour"));
    for forbidden in [
        "<script",
        "<iframe",
        "<object",
        "<embed",
        "<form",
        "<base",
        "<meta",
        "onclick",
        "onmouseover",
        "javascript:",
        "alert(1)",
    ] {
        assert!(!html.contains(forbidden), "{forbidden} dans {html}");
    }
}

/// Styles, classes et attributs de mise en page des tableaux sont conservés
#[test]
fn styles_and_layout_attributes_are_kept() {
    let input = concat!(
        r##"<style>.a{color:red}</style><table bgcolor="#fff" width="600" align="center" cellpadding="0">"##,
        r#"<tr><td class="a" style="padding:4px">x</td></tr></table><center>c</center>"#,
    );
    let html = sanitize_original_html(input);
    for kept in [
        "<style>.a{color:red}</style>",
        r##"bgcolor="#fff""##,
        r#"width="600""#,
        r#"align="center""#,
        r#"class="a""#,
        r#"style="padding:4px""#,
        "<center>",
    ] {
        assert!(html.contains(kept), "{kept} absent de {html}");
    }
}

/// Les images incorporées et distantes restent (la politique de contenu de la vue décide du chargement)
#[test]
fn images_are_kept_for_the_content_policy_to_decide() {
    let html = sanitize_original_html(r#"<img src="cid:logo@x"><img src="https://t.test/p.png">"#);
    assert!(html.contains("cid:logo@x") && html.contains("https://t.test/p.png"));
}

/// Un contenu piégé dans un style ou un SVG ne laisse passer aucun script
#[test]
fn style_and_svg_tricks_do_not_leave_script() {
    let input = r#"<style></style><script>a()</script><svg><animate onbegin="a()"/><script>b()</script></svg><math><mi xlink:href="javascript:a()">x</mi></math>"#;
    let html = sanitize_original_html(input);
    for forbidden in ["<script", "onbegin", "javascript:", "a()", "b()"] {
        assert!(!html.contains(forbidden), "{forbidden} dans {html}");
    }
}
