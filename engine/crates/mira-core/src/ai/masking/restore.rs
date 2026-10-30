//! Restitution en flux : les repères créés pour la requête redeviennent les valeurs d'origine.

use std::collections::BTreeMap;
use std::ops::Bound;

/// Résultat de la lecture d'un texte qui commence par `[`.
enum Bracket {
    Known(String, usize),
    Pending,
    Plain,
}

/// Remet les valeurs à la place des repères connus, même coupés entre deux morceaux de réponse.
pub struct Restorer {
    pub(super) table: BTreeMap<String, String>,
    pub(super) longest: usize,
    pub(super) pending: String,
}

impl Restorer {
    /// Crée le restituteur d'une table de repères ; une table vide laisse tout passer.
    pub(super) fn new(table: Vec<(String, String)>) -> Self {
        let longest = table.iter().map(|(p, _)| p.len()).max().unwrap_or(0);
        let keyed = table
            .into_iter()
            .map(|(placeholder, value)| (placeholder.to_ascii_uppercase(), value));
        Self {
            table: keyed.collect(),
            longest,
            pending: String::new(),
        }
    }

    /// Rend ce qui est sûr à émettre ; garde le suffixe qui peut encore devenir un repère connu.
    pub fn push(&mut self, chunk: &str) -> String {
        let text = std::mem::take(&mut self.pending) + chunk;
        let mut restored = String::with_capacity(text.len());
        let mut rest = text.as_str();
        while let Some(at) = rest.find('[') {
            restored.push_str(rest.get(..at).unwrap_or_default());
            rest = rest.get(at..).unwrap_or_default();
            match self.read_bracket(rest) {
                Bracket::Known(value, length) => {
                    restored.push_str(&value);
                    rest = rest.get(length..).unwrap_or_default();
                }
                Bracket::Pending => {
                    self.pending = rest.to_string();
                    return restored;
                }
                Bracket::Plain => {
                    restored.push('[');
                    rest = rest.get(1..).unwrap_or_default();
                }
            }
        }
        restored.push_str(rest);
        restored
    }

    /// Vide l'attente : à la fin du flux, ce qui ressemblait à un début de repère sort tel quel.
    pub fn finish(&mut self) -> String {
        std::mem::take(&mut self.pending)
    }

    /// Octets gardés en attente, jamais plus que le plus long repère.
    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    /// Dit si le texte est un repère connu, le début d'un repère connu, ou un simple crochet ; la casse ne compte pas.
    fn read_bracket(&self, rest: &str) -> Bracket {
        let window = rest.as_bytes().get(..self.longest.min(rest.len()));
        let closing = window.and_then(|bytes| bytes.iter().position(|byte| *byte == b']'));
        match closing {
            Some(at) => {
                let candidate = rest.get(..=at).unwrap_or_default();
                match self.table.get(&candidate.to_ascii_uppercase()) {
                    Some(value) => Bracket::Known(value.clone(), candidate.len()),
                    None => Bracket::Plain,
                }
            }
            None if rest.len() < self.longest && self.is_known_prefix(rest) => Bracket::Pending,
            None => Bracket::Plain,
        }
    }

    /// Dit si ce texte est le début, à la casse près, d'un repère connu.
    fn is_known_prefix(&self, text: &str) -> bool {
        let upper = text.to_ascii_uppercase();
        self.table
            .range::<str, _>((Bound::Included(upper.as_str()), Bound::Unbounded))
            .next()
            .is_some_and(|(placeholder, _)| placeholder.starts_with(&upper))
    }
}
