//! Variables connues d'un modèle et leur valeur dans un contexte de rendu.

use mira_vocabulary::RenderContext;

/// Variable qu'un modèle peut contenir entre doubles accolades.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateVariable {
    ContactName,
    ContactFirstName,
    ContactEmail,
    Date,
    Time,
    MyName,
    MyEmail,
    Subject,
}

impl TemplateVariable {
    /// Reconnaît une variable par son nom (sans accolades ni espaces de bord)
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "contact.nom" => Some(Self::ContactName),
            "contact.prénom" => Some(Self::ContactFirstName),
            "contact.email" => Some(Self::ContactEmail),
            "date" => Some(Self::Date),
            "heure" => Some(Self::Time),
            "moi.nom" => Some(Self::MyName),
            "moi.email" => Some(Self::MyEmail),
            "objet" => Some(Self::Subject),
            _ => None,
        }
    }

    /// Valeur dans le contexte ; une valeur absente ou vide n'en est pas une
    pub fn value(self, context: &RenderContext) -> Option<&str> {
        let contact = context.contact.as_ref();
        let found = match self {
            Self::ContactName => contact.and_then(|c| c.name.as_deref()),
            Self::ContactFirstName => contact.and_then(|c| c.first_name.as_deref()),
            Self::ContactEmail => contact.and_then(|c| c.email.as_deref()),
            Self::Date => context.date.as_deref(),
            Self::Time => context.time.as_deref(),
            Self::MyName => context.account_name.as_deref(),
            Self::MyEmail => context.account_email.as_deref(),
            Self::Subject => context.subject.as_deref(),
        };
        found.filter(|value| !value.is_empty())
    }
}
