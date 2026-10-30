//! Consommation d'une génération : total de tous les tours, delta remis au lecteur, jetons du journal.

use mira_vocabulary::ChatDelta;

use super::progress::Progress;

impl Progress {
    /// Consommation agrégée à remettre avant la fin, si le service en a annoncé une.
    pub(super) fn usage_delta(&self) -> Option<ChatDelta> {
        self.total()
            .map(|(input_tokens, output_tokens)| ChatDelta::Usage {
                input_tokens,
                output_tokens,
            })
    }

    /// Jetons d'entrée et de sortie pour le journal, inconnus si le service n'a rien annoncé.
    pub(super) fn tokens(&self) -> (Option<u32>, Option<u32>) {
        self.total().unzip()
    }

    /// Consommation de tous les tours : ceux qui sont clos plus celui en cours, inconnue si rien n'a été annoncé.
    fn total(&self) -> Option<(u32, u32)> {
        let both = |first: (u32, u32), second: (u32, u32)| {
            (
                first.0.saturating_add(second.0),
                first.1.saturating_add(second.1),
            )
        };
        match (self.spent, self.usage) {
            (None, None) => None,
            (spent, now) => Some(both(spent.unwrap_or_default(), now.unwrap_or_default())),
        }
    }
}
