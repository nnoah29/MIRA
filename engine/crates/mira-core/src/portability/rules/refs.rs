//! Correspondance entre identifiants locaux et références en clair, fournie par le stockage.

use mira_vocabulary::{AccountId, LocalLabelId, MailboxId, TemplateId};

/// Résolution des comptes ; `None` quand la cible n'existe pas
pub trait AccountRefs {
    /// Adresse du compte
    fn account_address(&self, id: &AccountId) -> Option<String>;
    /// Compte portant cette adresse
    fn account_by_address(&self, address: &str) -> Option<AccountId>;
}

/// Résolution des boîtes, désignées par adresse de compte et nom serveur
pub trait MailboxRefs {
    /// Adresse du compte et nom serveur de la boîte
    fn mailbox_ref(&self, id: &MailboxId) -> Option<(String, String)>;
    /// Boîte désignée par adresse de compte et nom serveur
    fn mailbox_by_ref(&self, address: &str, name: &str) -> Option<MailboxId>;
}

/// Résolution des libellés locaux, casse et espaces de bord ignorés
pub trait LabelRefs {
    /// Nom du libellé local
    fn label_name(&self, id: &LocalLabelId) -> Option<String>;
    /// Libellé local portant ce nom
    fn label_by_name(&self, name: &str) -> Option<LocalLabelId>;
}

/// Résolution des modèles de message, casse et espaces de bord ignorés
pub trait TemplateRefs {
    /// Nom du modèle de message
    fn template_name(&self, id: &TemplateId) -> Option<String>;
    /// Modèle portant ce nom
    fn template_by_name(&self, name: &str) -> Option<TemplateId>;
}

/// Toutes les résolutions nécessaires à la conversion d'une règle
pub trait RuleRefs: AccountRefs + MailboxRefs + LabelRefs + TemplateRefs {}

impl<T: AccountRefs + MailboxRefs + LabelRefs + TemplateRefs> RuleRefs for T {}
