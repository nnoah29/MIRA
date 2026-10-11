//! Implémentation de l'interface provider pour yahoo : données pures, sans état.

use mira_vocabulary::{
    EmailAddress, MailboxRole, Provider, ProviderDescriptor, RawMailbox, RawMessageAttributes,
    ServerMessageTranslation, ServerSettings,
};

use crate::descriptor::{DOMAINS, descriptor};
use crate::roles::role_for_name;
use crate::servers::servers;

/// Provider yahoo (structure unitaire, sans état).
#[derive(Debug, Clone, Copy, Default)]
pub struct YahooProvider;

impl Provider for YahooProvider {
    /// Décrit le service : domaines, mot de passe d'application, limites, dossiers.
    fn descriptor(&self) -> ProviderDescriptor {
        descriptor()
    }

    /// Reconnaît exactement les domaines du service ; un sous-domaine n'en fait pas partie.
    fn handles_address(&self, address: &EmailAddress) -> bool {
        DOMAINS.contains(&address.domain())
    }

    /// Donne les serveurs fixes du service.
    fn servers_for(&self, _address: &EmailAddress) -> ServerSettings {
        servers()
    }

    /// Reconnaît les noms de dossiers propres au service.
    fn mailbox_role_hint(&self, mailbox: &RawMailbox) -> Option<MailboxRole> {
        role_for_name(&mailbox.name)
    }

    /// N'a aucun attribut propre à traduire.
    fn translate_attributes(&self, _raw: &RawMessageAttributes) -> ServerMessageTranslation {
        ServerMessageTranslation::default()
    }
}
