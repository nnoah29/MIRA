//! Restitution en flux : les repères créés pour la requête redeviennent les valeurs d'origine.

/// Résultat de la lecture d'un texte qui commence par `[`.
enum Bracket {
    Known(usize),
    Pending,
    Plain,
}

/// Remet les valeurs à la place des repères connus, même coupés entre deux morceaux de réponse.
pub struct Restorer {
    pub(super) table: Vec<(String, String)>,
    pub(super) pending: String,
}

impl Restorer {
    /// Crée le restituteur d'une table de repères ; une table vide laisse tout passer.
    pub(super) fn new(table: Vec<(String, String)>) -> Self {
        Self {
            table,
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
                Bracket::Known(index) => {
                    let (placeholder, value) = self.table.get(index).cloned().unwrap_or_default();
                    restored.push_str(&value);
                    rest = rest.get(placeholder.len()..).unwrap_or_default();
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

    /// Dit si le texte est un repère connu, le début d'un repère connu, ou un simple crochet.
    fn read_bracket(&self, rest: &str) -> Bracket {
        let known = self
            .table
            .iter()
            .position(|(placeholder, _)| rest.starts_with(placeholder.as_str()));
        match known {
            Some(index) => Bracket::Known(index),
            None if self.table.iter().any(|(p, _)| p.starts_with(rest)) => Bracket::Pending,
            None => Bracket::Plain,
        }
    }
}
