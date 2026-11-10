#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Ligne de commande du serveur local : jetons remplacés, raisonnement coupé, threads bornés.

use std::path::PathBuf;

use mira_core::ai::local::{ArgValues, build_args, thread_count};

mod local_support;
use local_support::test_runtime;

/// Valeurs de jetons de test
fn values() -> ArgValues {
    ArgValues {
        model: PathBuf::from("/m/model.bin"),
        host: "127.0.0.1".into(),
        port: 4242,
        api_key_file: PathBuf::from("/run/api-key"),
        threads: 6,
    }
}

/// Chaque jeton est remplacé par sa valeur et la clé n'apparaît que comme chemin de fichier
#[test]
fn tokens_are_replaced_and_the_key_is_only_a_file_path() {
    let runtime = test_runtime();
    let model = runtime.models[0].clone();
    let args = build_args(&runtime, &model, &values()).unwrap();
    assert!(args.windows(2).any(|w| w == ["--port", "4242"]));
    assert!(args.windows(2).any(|w| w == ["--model", "/m/model.bin"]));
    assert!(args.iter().all(|a| !a.contains('{')));
}

/// Les arguments de raisonnement coupé ne sont ajoutés que si le modèle ne raisonne pas
#[test]
fn args_add_reasoning_off_only_when_the_model_does_not_reason() {
    let mut runtime = test_runtime();
    runtime.reasoning_off_args = vec!["--reasoning".into(), "off".into()];
    let mut model = runtime.models[0].clone();
    model.reasoning = false;
    assert!(
        build_args(&runtime, &model, &values())
            .unwrap()
            .ends_with(&["--reasoning".into(), "off".into()])
    );
    model.reasoning = true;
    assert!(
        !build_args(&runtime, &model, &values())
            .unwrap()
            .contains(&"--reasoning".to_string())
    );
}

/// Un jeton inconnu est une erreur, jamais un argument laissé tel quel
#[test]
fn an_unknown_token_is_an_error() {
    let mut runtime = test_runtime();
    runtime.server_args.push("{secret}".into());
    let model = runtime.models[0].clone();
    assert!(build_args(&runtime, &model, &values()).is_err());
}

/// Les threads sont les cœurs moins deux, au moins deux, jamais plus que les cœurs
#[test]
fn thread_count_is_cores_minus_two_bounded() {
    assert_eq!(thread_count(8), 6);
    assert_eq!(thread_count(4), 2);
    assert_eq!(thread_count(2), 2);
    assert_eq!(thread_count(1), 1);
}
