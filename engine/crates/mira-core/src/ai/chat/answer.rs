//! Texte de la réponse : repères restitués au fil du flux, et annonce unique du nombre de valeurs masquées.

use mira_vocabulary::ChatDelta;

use crate::ai::masking::{Masker, Restorer};

/// Restitue les valeurs masquées dans la réponse et annonce le masquage avant le premier texte.
#[derive(Debug)]
pub(super) struct Answer {
    restorer: Restorer,
    masked: u32,
    announced: bool,
}

impl Answer {
    /// Prend la table du masqueur de la requête, qui ne vit que le temps de la génération.
    pub(super) fn new(masker: Masker) -> Self {
        Self {
            masked: masker.count(),
            restorer: masker.into_restorer(),
            announced: false,
        }
    }

    /// Fait passer un texte décodé par la restitution et rend les deltas à transmettre.
    pub(super) fn push(&mut self, text: &str) -> Vec<ChatDelta> {
        let restored = self.restorer.push(text);
        self.emit(restored)
    }

    /// Vide ce que la restitution gardait en attente, à la fin du flux.
    pub(super) fn flush(&mut self) -> Vec<ChatDelta> {
        let rest = self.restorer.finish();
        self.emit(rest)
    }

    /// Nombre de valeurs masquées dans la requête, pour le journal.
    pub(super) fn masked(&self) -> u32 {
        self.masked
    }

    /// Texte restitué en delta, précédé de l'annonce du masquage la première fois ; rien si le texte est vide.
    fn emit(&mut self, restored: String) -> Vec<ChatDelta> {
        if restored.is_empty() {
            return Vec::new();
        }
        let mut deltas = Vec::with_capacity(2);
        if self.masked > 0 && !self.announced {
            self.announced = true;
            deltas.push(ChatDelta::Masked { count: self.masked });
        }
        deltas.push(ChatDelta::Text { text: restored });
        deltas
    }
}
