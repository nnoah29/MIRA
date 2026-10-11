//! Implémentation de l'interface provider pour le repli : toujours applicable, sans particularité.

use mira_vocabulary::{
    EmailAddress, MailboxRole, Provider, ProviderDescriptor, RawMailbox, RawMessageAttributes,
    ServerMessageTranslation, ServerSettings,
};

use crate::descriptor::descriptor;
use crate::servers::servers_for_domain;

/// Provider de repli pour tout service inconnu (structure unitaire, sans état).
#[derive(Debug, Clone, Copy, Default)]
pub struct GenericProvider;

impl Provider for GenericProvider {
    /// Décrit le repli : aucun domaine, mot de passe ou mot de passe d'application.
    fn descriptor(&self) -> ProviderDescriptor {
        descriptor()
    }

    /// Accepte toute adresse, puisque c'est le repli.
    fn handles_address(&self, _address: &EmailAddress) -> bool {
        true
    }

    /// Devine les serveurs `imap.<domaine>` et `smtp.<domaine>`.
    fn servers_for(&self, address: &EmailAddress) -> ServerSettings {
        servers_for_domain(address.domain())
    }

    /// N'a aucun indice de rôle : le cœur applique la règle générale.
    fn mailbox_role_hint(&self, _mailbox: &RawMailbox) -> Option<MailboxRole> {
        None
    }

    /// N'a aucun attribut propre à traduire.
    fn translate_attributes(&self, _raw: &RawMessageAttributes) -> ServerMessageTranslation {
        ServerMessageTranslation::default()
    }
}
