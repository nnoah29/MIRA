//! Assemblage des exemples des domaines d'écriture et nommage des exemples multiples.

use std::collections::BTreeMap;

use mira_core::api::{Request, Response};

use super::{Named, actions, compose, queue};

/// Requêtes d'exemple des domaines d'écriture (actions, rédaction, file d'actions)
pub fn write_requests() -> Vec<Named<Request>> {
    let mut all = actions::requests();
    all.extend(compose::requests());
    all.extend(queue::requests());
    all
}

/// Réponses d'exemple de l'enveloppe d'écriture
pub fn write_responses() -> Vec<Named<Response>> {
    let mut all = actions::responses();
    all.extend(compose::responses());
    all.extend(queue::responses());
    all
}

/// Noms uniques `domaine.opération` (`.2`, `.3`… pour les exemples suivants d'une même opération)
pub fn numbered(domain: &str, ops: &[&str]) -> Vec<String> {
    let mut seen: BTreeMap<String, u32> = BTreeMap::new();
    let mut names = Vec::new();
    for op in ops {
        let count = seen.entry((*op).to_string()).or_insert(0);
        *count += 1;
        let name = match *count {
            1 => format!("{domain}.{op}"),
            rank => format!("{domain}.{op}.{rank}"),
        };
        names.push(name);
    }
    names
}
