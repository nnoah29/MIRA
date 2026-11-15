//! Outil `recall_memory` : retrouve par le sens les souvenirs de l'utilisateur ; contenu de type « mémoire », soumis au consentement.

use mira_vocabulary::{MemoryHit, MemoryKind, ToolSpec};
use serde::Deserialize;

use super::args::parse;
use super::run_result::ToolRun;
use crate::api::Engine;

pub(super) const NAME: &str = "recall_memory";

const MAX_HITS: usize = 5;
const MAX_QUERY_CHARS: usize = 1_000;

const SCHEMA: &str = r#"{"type":"object","properties":{
"query":{"type":"string","description":"Ce que tu cherches dans la mémoire de l'utilisateur, en langage naturel."}
},"required":["query"]}"#;

/// Arguments de la recherche.
#[derive(Debug, Deserialize)]
struct RecallArgs {
    query: String,
}

/// Fiche de l'outil annoncée au modèle.
pub(super) fn spec() -> ToolSpec {
    ToolSpec {
        name: NAME.to_string(),
        description: "Cherche dans la mémoire de l'utilisateur (faits, préférences, actions passées) les souvenirs proches d'une question ; chaque souvenir porte son identifiant, utile pour proposer de l'oublier.".to_string(),
        parameters_schema: SCHEMA.to_string(),
    }
}

/// Étiquette du genre de souvenir, dite au modèle.
fn kind_label(kind: MemoryKind) -> &'static str {
    match kind {
        MemoryKind::Fact => "fait",
        MemoryKind::Preference => "préférence",
        MemoryKind::Action => "action",
    }
}

/// Une ligne par souvenir : identifiant, genre et texte.
pub(crate) fn render_hits(hits: &[MemoryHit]) -> String {
    let lines: Vec<String> = hits
        .iter()
        .map(|hit| {
            let record = &hit.record;
            format!(
                "- id={} ({}) : {}",
                record.id,
                kind_label(record.kind),
                record.text
            )
        })
        .collect();
    format!(
        "{} souvenir(s) proche(s) :\n{}",
        hits.len(),
        lines.join("\n")
    )
}

/// Cherche et rend les souvenirs ; une panne est expliquée sans détail ni contenu.
pub(super) async fn run(engine: &Engine, arguments: &str) -> ToolRun {
    let args: RecallArgs = match parse(arguments) {
        Ok(args) => args,
        Err(text) => return ToolRun::error(text),
    };
    let query: String = args.query.trim().chars().take(MAX_QUERY_CHARS).collect();
    if query.is_empty() {
        return ToolRun::error("la requête est vide".to_string());
    }
    let Ok(store) = engine.memory_store().await else {
        return ToolRun::error("la mémoire n'est pas disponible".to_string());
    };
    match store.recall(&query, MAX_HITS).await {
        Ok(hits) if hits.is_empty() => {
            ToolRun::recalled("Aucun souvenir ne correspond.".to_string(), 0)
        }
        Ok(hits) => ToolRun::recalled(
            render_hits(&hits),
            u32::try_from(hits.len()).unwrap_or(u32::MAX),
        ),
        Err(_) => ToolRun::error("la recherche dans la mémoire a échoué".to_string()),
    }
}
