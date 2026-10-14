//! Exemples du contrat : une requête par variante de `Request` et une réponse par forme, groupés par domaine.
#![allow(dead_code)]

pub mod actions;
pub mod compose;
pub mod compose_drafts;
pub mod compose_sending;
pub mod queue;

use std::collections::BTreeMap;

use mira_core::api::{Request, Response};

/// Un exemple nommé (`domaine.opération`, suivi d'un rang si l'opération a plusieurs exemples).
pub type Named<T> = (String, T);

/// Toutes les requêtes d'exemple, tous domaines confondus
pub fn all_requests() -> Vec<Named<Request>> {
    let mut all = actions::requests();
    all.extend(compose::requests());
    all.extend(queue::requests());
    all
}

/// Toutes les réponses d'exemple, tous domaines confondus
pub fn all_responses() -> Vec<Named<Response>> {
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
