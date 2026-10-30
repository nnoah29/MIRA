//! Ce que la génération a demandé aux outils : noms et nombres d'appels, types réellement envoyés, caractères.

use mira_vocabulary::{AiToolCount, ContentKind};

use crate::ai::tools::kind_of;

/// Bilan des outils d'une génération, pour la ligne de journal ; jamais d'argument ni de contenu.
#[derive(Debug, Default)]
pub(super) struct ToolUsage {
    pub(super) counts: Vec<AiToolCount>,
    pub(super) kinds: Vec<ContentKind>,
    pub(super) chars: u32,
}

impl ToolUsage {
    /// Note un appel : son nom (`inconnu` si ce n'est pas un outil du catalogue), le type envoyé et la taille envoyée.
    pub(super) fn record(&mut self, name: &str, sent: Option<ContentKind>, chars: usize) {
        let name = if kind_of(name).is_some() {
            name
        } else {
            "unknown"
        };
        match self.counts.iter_mut().find(|count| count.name == name) {
            Some(count) => count.count += 1,
            None => self.counts.push(AiToolCount {
                name: name.to_string(),
                count: 1,
            }),
        }
        if let Some(kind) = sent.filter(|kind| !self.kinds.contains(kind)) {
            self.kinds.push(kind);
        }
        self.chars = self
            .chars
            .saturating_add(u32::try_from(chars).unwrap_or(u32::MAX));
    }
}
