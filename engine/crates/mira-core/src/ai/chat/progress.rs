//! Ce que la tâche retient du flux : première raison d'arrêt, consommation maximale, texte reçu.

use mira_vocabulary::{AiError, ChatDelta, StopReason};

use super::ending::Ending;

/// État accumulé au fil des deltas décodés.
#[derive(Debug, Default)]
pub(super) struct Progress {
    stop: Option<StopReason>,
    usage: Option<(u32, u32)>,
    text_seen: bool,
}

impl Progress {
    /// Retient un delta décodé et rend celui à transmettre tout de suite (le texte seulement).
    pub(super) fn absorb(&mut self, delta: ChatDelta) -> Option<ChatDelta> {
        match delta {
            ChatDelta::Text { .. } => {
                self.text_seen = true;
                Some(delta)
            }
            ChatDelta::Usage {
                input_tokens,
                output_tokens,
            } => {
                let (input, output) = self.usage.unwrap_or((0, 0));
                self.usage = Some((input.max(input_tokens), output.max(output_tokens)));
                None
            }
            ChatDelta::Masked { .. } => None,
            ChatDelta::Done { reason } => {
                self.stop.get_or_insert(reason);
                None
            }
        }
    }

    /// Issue d'un flux fini : première raison décodée, sinon fin normale s'il y a eu du texte.
    pub(super) fn end_of_stream(&self) -> Ending {
        match (&self.stop, self.text_seen) {
            (Some(reason), _) => Ending::Finished(reason.clone()),
            (None, true) => Ending::Finished(StopReason::EndTurn),
            (None, false) => Ending::Failed(AiError::BadResponse),
        }
    }

    /// Consommation agrégée à remettre avant la fin, si le service en a annoncé une.
    pub(super) fn usage_delta(&self) -> Option<ChatDelta> {
        self.usage
            .map(|(input_tokens, output_tokens)| ChatDelta::Usage {
                input_tokens,
                output_tokens,
            })
    }

    /// Jetons d'entrée et de sortie pour le journal, inconnus si le service n'a rien annoncé.
    pub(super) fn tokens(&self) -> (Option<u32>, Option<u32>) {
        self.usage.unzip()
    }
}
