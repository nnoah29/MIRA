//! Ce que la tâche retient du flux : première raison d'arrêt, consommation maximale, texte reçu.

use mira_vocabulary::{AiError, ChatDelta, StopReason};

use super::answer::Answer;
use super::assembler::ToolAssembler;
use super::ending::Ending;
use super::tool_usage::ToolUsage;

/// État accumulé au fil des deltas décodés.
#[derive(Debug)]
pub(super) struct Progress {
    pub(super) stop: Option<StopReason>,
    pub(super) usage: Option<(u32, u32)>,
    pub(super) spent: Option<(u32, u32)>,
    pub(super) text_seen: bool,
    pub(super) turn_text: String,
    pub(super) tools: ToolAssembler,
    pub(super) used: ToolUsage,
    pub(super) answer: Answer,
}

impl Progress {
    /// Démarre sans rien avoir reçu, avec la restitution de la réponse de cette requête.
    pub(super) fn new(answer: Answer) -> Self {
        Self {
            stop: None,
            usage: None,
            spent: None,
            text_seen: false,
            turn_text: String::new(),
            tools: ToolAssembler::default(),
            used: ToolUsage::default(),
            answer,
        }
    }

    /// Retient un delta décodé et rend ceux à transmettre tout de suite (le texte restitué seulement).
    pub(super) fn absorb(&mut self, delta: ChatDelta) -> Vec<ChatDelta> {
        match delta {
            ChatDelta::Text { text } => self.absorb_text(&text),
            ChatDelta::Usage {
                input_tokens,
                output_tokens,
            } => self.absorb_usage(input_tokens, output_tokens),
            ChatDelta::ToolFragment {
                index,
                id,
                name,
                arguments,
            } => {
                self.tools.push(index, id, name, &arguments);
                Vec::new()
            }
            ChatDelta::Done { reason } => {
                self.stop.get_or_insert(reason);
                Vec::new()
            }
            ChatDelta::Masked { .. }
            | ChatDelta::ToolCalled { .. }
            | ChatDelta::ToolFinished { .. } => Vec::new(),
        }
    }

    /// Retient le texte du tour tel qu'écrit et rend ce que la restitution laisse sortir.
    fn absorb_text(&mut self, text: &str) -> Vec<ChatDelta> {
        self.text_seen = true;
        self.turn_text.push_str(text);
        self.answer.push(text)
    }

    /// Retient la plus grande consommation annoncée pendant ce tour.
    fn absorb_usage(&mut self, input_tokens: u32, output_tokens: u32) -> Vec<ChatDelta> {
        let (input, output) = self.usage.unwrap_or((0, 0));
        self.usage = Some((input.max(input_tokens), output.max(output_tokens)));
        Vec::new()
    }

    /// Issue d'un flux fini : première raison décodée, sinon fin normale s'il y a eu du texte.
    pub(super) fn end_of_stream(&self) -> Ending {
        match (&self.stop, self.text_seen) {
            (Some(reason), _) => Ending::Finished(reason.clone()),
            (None, true) => Ending::Finished(StopReason::EndTurn),
            (None, false) => Ending::Failed(AiError::BadResponse),
        }
    }
}
