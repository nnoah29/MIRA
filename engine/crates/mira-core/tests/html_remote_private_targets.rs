#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Images distantes : même autorisées, jamais vers une machine locale ou privée (audit SEC, A10-3).

use mira_core::html::{sanitize_html, sanitize_original_html};

const PRIVATE_TARGETS: [&str; 8] = [
    "http://127.0.0.1:8080/pixel.png",
    "https://192.168.1.1/admin?reboot=1",
    "http://10.0.0.5/x.gif",
    "http://169.254.169.254/latest/meta-data",
    "http://localhost/x.png",
    "http://routeur.local/x.png",
    "http://[::1]/x.png",
    "http://intranet/logo.png",
];

/// Image vers l'adresse donnée
fn image(src: &str) -> String {
    format!("<p>a</p><img src=\"{src}\"><p>b</p>")
}

/// Avec le contenu distant autorisé, une image vers une machine privée est retirée et signalée
#[test]
fn allowed_remote_content_still_refuses_private_targets() {
    for src in PRIVATE_TARGETS {
        let safe = sanitize_html(&image(src), true);
        assert!(!safe.html.contains("src="), "{src} : {}", safe.html);
        assert!(safe.blocked_resources.iter().any(|b| b == src), "{src}");
    }
}

/// Une image publique reste servie quand le contenu distant est autorisé
#[test]
fn allowed_remote_content_keeps_public_images() {
    let src = "https://exemple.org/logo.png";
    assert!(sanitize_html(&image(src), true).html.contains(src));
    assert!(sanitize_original_html(&image(src)).contains(src));
}

/// Un `srcset` dont une seule entrée vise une machine privée est retiré en entier
#[test]
fn srcset_with_one_private_entry_is_dropped() {
    let html = "<img srcset=\"https://exemple.org/a.png 1x, http://127.0.0.1/b.png 2x\">";
    assert!(!sanitize_html(html, true).html.contains("srcset"));
    assert!(!sanitize_original_html(html).contains("srcset"));
}

/// Le HTML d'origine (vue web) perd aussi les cibles privées, et l'adresse relative au protocole
#[test]
fn original_html_drops_private_targets_too() {
    for src in PRIVATE_TARGETS {
        let cleaned = sanitize_original_html(&image(src));
        assert!(!cleaned.contains("src="), "{src} : {cleaned}");
    }
    let relative = sanitize_original_html(&image("//127.0.0.1/x.png"));
    assert!(!relative.contains("src="), "{relative}");
}
