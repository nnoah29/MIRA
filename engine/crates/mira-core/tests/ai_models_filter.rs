#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Liste des modèles : exclusion par mot et tri par date de création puis par ordre alphabétique.

use mira_core::ai::models::{filter_models, sort_models};
use mira_vocabulary::AiModel;

/// Modèle de test avec sa date de création éventuelle
fn model(id: &str, created_at: Option<i64>) -> AiModel {
    AiModel {
        id: id.to_string(),
        created_at,
    }
}

/// Les identifiants d'une liste, dans l'ordre
fn ids(models: &[AiModel]) -> Vec<&str> {
    models.iter().map(|item| item.id.as_str()).collect()
}

/// Un mot exclu écarte tout identifiant qui le contient, sans tenir compte de la casse
#[test]
fn filter_excludes_by_word_case_insensitive() {
    let models = vec![
        model("gpt-chat", None),
        model("Text-EMBEDDING-3", None),
        model("whisper-1", None),
        model("tts-hd", None),
    ];
    let exclusions = vec!["embedding".to_string(), "WHISPER".to_string()];
    let kept = filter_models(models, &exclusions);
    assert_eq!(ids(&kept), ["gpt-chat", "tts-hd"]);
}

/// Sans exclusion, tout est gardé dans l'ordre reçu
#[test]
fn filter_keeps_everything_with_empty_exclusions() {
    let models = vec![model("b", None), model("a", Some(1))];
    assert_eq!(filter_models(models.clone(), &[]), models);
}

/// Les plus récents d'abord ; sans date, après ceux qui en ont, par ordre alphabétique
#[test]
fn sort_by_created_desc_then_alpha_for_missing() {
    let models = vec![
        model("zz-nodate", None),
        model("old", Some(100)),
        model("aa-nodate", None),
        model("new", Some(300)),
        model("mid", Some(200)),
    ];
    let sorted = sort_models(models);
    assert_eq!(
        ids(&sorted),
        ["new", "mid", "old", "aa-nodate", "zz-nodate"]
    );
}

/// À date égale, l'ordre alphabétique départage ; une liste sans date est alphabétique
#[test]
fn sort_ties_and_all_undated_are_alphabetical() {
    let tied = sort_models(vec![model("b", Some(5)), model("a", Some(5))]);
    assert_eq!(ids(&tied), ["a", "b"]);
    let undated = sort_models(vec![model("c", None), model("a", None), model("b", None)]);
    assert_eq!(ids(&undated), ["a", "b", "c"]);
}
