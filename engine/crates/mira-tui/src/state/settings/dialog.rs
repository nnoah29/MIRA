//! Fenêtre de la vue Paramètres : formulaire de texte ou confirmation, une seule à la fois.

use mira_assembly::mira_vocabulary::{AccountId, IdentityId};

/// Ce que la vue Paramètres affiche par-dessus ses pages, s'il y a quelque chose.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Dialog {
    #[default]
    None,
    Form(Form),
    Confirm(Ask),
}

/// Opération de maintenance qui demande une confirmation explicite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ask {
    PurgeCache,
    RebuildIndex,
}

/// Nature d'un export de maintenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    Diagnostic,
    Profile,
}

/// Ce que le formulaire ouvert modifie ; l'identifiant visé est pris à l'ouverture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormKind {
    EditAccount(AccountId),
    DeleteAccount { account: AccountId, address: String },
    AddIdentity(AccountId),
    EditIdentity(IdentityId),
    Export(ExportKind),
}

/// Un champ de texte : libellé, valeur courante et valeur d'origine (pour n'envoyer que les changements).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub label: &'static str,
    pub value: String,
    pub original: String,
}

/// Formulaire à champs de texte et champ qui a le focus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Form {
    pub kind: FormKind,
    pub fields: Vec<Field>,
    pub focus: usize,
}

impl Field {
    /// Champ dont la valeur d'origine est la valeur initiale.
    pub fn new(label: &'static str, value: &str) -> Field {
        Field {
            label,
            value: value.to_string(),
            original: value.to_string(),
        }
    }

    /// Vrai si la valeur a changé depuis l'ouverture.
    pub fn changed(&self) -> bool {
        self.value != self.original
    }
}

impl Form {
    /// Formulaire sur ses champs, le focus sur le premier.
    pub fn new(kind: FormKind, fields: Vec<Field>) -> Form {
        Form {
            kind,
            fields,
            focus: 0,
        }
    }

    /// Valeur du champ à cet indice, vide s'il n'existe pas.
    pub fn value(&self, index: usize) -> &str {
        self.fields.get(index).map_or("", |f| f.value.as_str())
    }

    /// Vrai si le champ à cet indice a changé.
    pub fn changed(&self, index: usize) -> bool {
        self.fields.get(index).is_some_and(Field::changed)
    }
}
