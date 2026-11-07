#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Choix de la commande qui relance l'app et de la langue du système.

use mira_daemon::desktop::{launch_plan, pick_locale};

/// Le binaire `mira` posé à côté du démon est préféré.
#[test]
fn the_app_next_to_the_daemon_is_preferred() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("mira");
    std::fs::write(&app, b"#!/bin/sh\n").unwrap();
    let (program, arguments) = launch_plan(Some(dir.path()));
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
    assert_eq!(launch_plan(Some(dir.path())), expected);
    assert_eq!(launch_plan(None), expected);
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
