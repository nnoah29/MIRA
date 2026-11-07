#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Création concurrente du jeton : un lecteur ne voit jamais un fichier vide ou à moitié écrit.

use std::sync::{Arc, Barrier};

use mira_daemon::token::Token;

/// Seize fils créent le même jeton en même temps, trente fois : aucun échec, un seul jeton.
#[test]
fn concurrent_creations_all_succeed_and_agree_on_one_token() {
    for round in 0..30 {
        let dir = tempfile::tempdir().unwrap();
        let path = Arc::new(dir.path().join(format!("round{round}/daemon.token")));
        let gate = Arc::new(Barrier::new(16));
        let workers: Vec<_> = (0..16)
            .map(|_| {
                let (path, gate) = (path.clone(), gate.clone());
                std::thread::spawn(move || {
                    gate.wait();
                    Token::load_or_create(&path).map(|t| t.as_str().to_string())
                })
            })
            .collect();
        let tokens: Vec<String> = workers
            .into_iter()
            .map(|w| w.join().unwrap().expect("aucun fil ne doit échouer"))
            .collect();
        assert!(tokens.iter().all(|t| t == &tokens[0]), "{tokens:?}");
    }
}

/// Aucun fichier temporaire ne reste à côté du jeton.
#[test]
fn no_temporary_file_is_left_beside_the_token() {
    let dir = tempfile::tempdir().unwrap();
    Token::load_or_create(&dir.path().join("daemon.token")).unwrap();
    let names: Vec<String> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["daemon.token"]);
}
