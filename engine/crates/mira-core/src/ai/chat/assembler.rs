//! Assemblage des appels d'outils reçus par fragments : identifiant et nom au premier, arguments concaténés par rang.

use std::collections::BTreeMap;

use mira_vocabulary::ToolCall;

const MAX_ARGUMENT_BYTES: usize = 65_536;

/// Un appel en cours de réception.
#[derive(Debug, Default)]
struct Partial {
    id: Option<String>,
    name: Option<String>,
    arguments: String,
    overflow: bool,
}

/// Appels du tour en cours, rangés par numéro de bloc ou d'index.
#[derive(Debug, Default)]
pub(super) struct ToolAssembler {
    calls: BTreeMap<u32, Partial>,
}

impl ToolAssembler {
    /// Ajoute un fragment ; des arguments trop longs sont abandonnés et l'appel sera refusé.
    pub(super) fn push(
        &mut self,
        index: u32,
        id: Option<String>,
        name: Option<String>,
        arguments: &str,
    ) {
        let partial = self.calls.entry(index).or_default();
        partial.id = partial.id.take().or(id);
        partial.name = partial.name.take().or(name);
        if partial.overflow {
            return;
        }
        if partial.arguments.len() + arguments.len() > MAX_ARGUMENT_BYTES {
            partial.overflow = true;
            partial.arguments.clear();
            return;
        }
        partial.arguments.push_str(arguments);
    }

    /// Rend les appels complets dans l'ordre et vide l'assembleur ; sans nom, un appel est écarté.
    pub(super) fn finish(&mut self) -> Vec<ToolCall> {
        let calls = std::mem::take(&mut self.calls);
        calls
            .into_iter()
            .filter_map(|(index, partial)| {
                let empty = partial.arguments.trim().is_empty() && !partial.overflow;
                Some(ToolCall {
                    id: partial.id.unwrap_or_else(|| format!("call_{index}")),
                    name: partial.name?,
                    arguments: if empty {
                        "{}".to_string()
                    } else {
                        partial.arguments
                    },
                })
            })
            .collect()
    }
}
