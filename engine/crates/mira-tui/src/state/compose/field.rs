//! Champs d'un brouillon, dans l'ordre de saisie.

/// Nombre de champs d'un brouillon.
pub const FIELD_COUNT: usize = 5;

/// Champ qui reçoit la frappe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    To,
    Cc,
    Bcc,
    Subject,
    Body,
}

/// Famille de champ : adresses, objet d'une ligne, corps de plusieurs lignes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Address,
    Subject,
    Body,
}

impl Field {
    /// Tous les champs, dans l'ordre de l'écran.
    pub const ALL: [Field; FIELD_COUNT] = [
        Field::To,
        Field::Cc,
        Field::Bcc,
        Field::Subject,
        Field::Body,
    ];

    /// Rang du champ dans l'ordre de l'écran.
    pub fn index(self) -> usize {
        match self {
            Field::To => 0,
            Field::Cc => 1,
            Field::Bcc => 2,
            Field::Subject => 3,
            Field::Body => 4,
        }
    }

    /// Champ suivant, en cycle.
    pub fn next(self) -> Field {
        Field::ALL[(self.index() + 1) % FIELD_COUNT]
    }

    /// Champ précédent, en cycle.
    pub fn previous(self) -> Field {
        Field::ALL[(self.index() + FIELD_COUNT - 1) % FIELD_COUNT]
    }

    /// Nom affiché devant le champ.
    pub fn label(self) -> &'static str {
        match self {
            Field::To => "À",
            Field::Cc => "Cc",
            Field::Bcc => "Cci",
            Field::Subject => "Objet",
            Field::Body => "Corps",
        }
    }

    /// Famille du champ, qui décide de la façon de réagir aux touches.
    pub fn kind(self) -> FieldKind {
        match self {
            Field::To | Field::Cc | Field::Bcc => FieldKind::Address,
            Field::Subject => FieldKind::Subject,
            Field::Body => FieldKind::Body,
        }
    }
}
