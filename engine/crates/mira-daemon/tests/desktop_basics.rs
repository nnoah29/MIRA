#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Choix de la commande qui relance l'app et de la langue du système.

use mira_daemon::desktop::{body_text, launch_plan, pick_locale};

/// Le binaire `mira` posé à côté du démon est préféré.
#[test]
fn the_app_next_to_the_daemon_is_preferred() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("mira");
    std::fs::write(&app, b"#!/bin/sh\n").unwrap();
    let (program, arguments) = launch_plan(Some(dir.path()), None);
    assert_eq!(program, app.to_string_lossy());
    assert!(arguments.is_empty());
}

/// Sans binaire voisin, ou sans dossier connu, on passe par le lanceur du système.
#[test]
fn without_a_neighbour_the_system_launcher_is_used() {
    let dir = tempfile::tempdir().unwrap();
    let expected = (
        "gtk-launch".to_string(),
        vec!["io.github.nnoah29.Mira".to_string()],
    );
    assert_eq!(launch_plan(Some(dir.path()), None), expected);
    assert_eq!(launch_plan(None, None), expected);
}

/// La première variable non vide l'emporte ; aucune donne une langue vide.
#[test]
fn the_first_non_empty_variable_wins() {
    assert_eq!(
        pick_locale(Some(""), Some("fr_FR.UTF-8"), Some("en_US")),
        "fr_FR.UTF-8"
    );
    assert_eq!(pick_locale(None, None, Some("de_DE")), "de_DE");
    assert_eq!(pick_locale(None, None, None), "");
}

/// `MIRA_APP_BIN` l'emporte s'il désigne un fichier ; sinon il est ignoré.
#[test]
fn an_explicit_app_wins_only_when_it_exists() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("autre");
    std::fs::write(&app, b"#!/bin/sh\n").unwrap();
    let plan = launch_plan(Some(dir.path()), Some(&app));
    assert_eq!(plan.0, app.to_string_lossy());
    let missing = dir.path().join("absent");
    assert_eq!(launch_plan(None, Some(&missing)).0, "gtk-launch");
}

/// Le corps est borné à 400 caractères et échappé seulement quand le service interprète le balisage.
#[test]
fn the_body_is_bounded_and_escaped_only_for_markup_services() {
    assert_eq!(body_text("a<b", true), "a&lt;b");
    assert_eq!(body_text("a<b", false), "a<b");
    assert_eq!(body_text(&"é".repeat(1000), false).chars().count(), 400);
}
