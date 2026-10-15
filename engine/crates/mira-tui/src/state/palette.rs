//! Palette de commandes `:` : saisie ouverte ou non et texte tapé.

const MAX_INPUT_CHARS: usize = 60;

/// Saisie de la palette ; fermée tant que l'utilisateur n'a pas tapé `:`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PaletteState {
    open: bool,
    input: String,
}

impl PaletteState {
    /// Ouvre une saisie vide.
    pub fn open(&mut self) {
        self.open = true;
        self.input.clear();
    }

    /// Ferme la saisie et l'efface.
    pub fn close(&mut self) {
        self.open = false;
        self.input.clear();
    }

    /// Vrai tant que la saisie est ouverte.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Texte tapé jusqu'ici.
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Ajoute un caractère imprimable, dans la limite de longueur.
    pub fn push(&mut self, c: char) {
        if !c.is_control() && self.input.chars().count() < MAX_INPUT_CHARS {
            self.input.push(c);
        }
    }

    /// Retire le dernier caractère tapé.
    pub fn pop(&mut self) {
        self.input.pop();
    }
}
