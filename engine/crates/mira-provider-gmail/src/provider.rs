//! Implémentation de l'interface provider pour gmail : données pures, sans état.

use mira_vocabulary::{
    EmailAddress, MailboxRole, Provider, ProviderDescriptor, RawMailbox, RawMessageAttributes,
    ServerMessageTranslation, ServerSettings,
};

use crate::descriptor::{DOMAINS, descriptor};
use crate::roles::role_for_name;
use crate::servers::servers;
use crate::translate::translate;

/// Provider gmail (structure unitaire, sans état).
#[derive(Debug, Clone, Copy, Default)]
pub struct GmailProvider;

impl Provider for GmailProvider {
    /// Décrit le service : domaines, OAuth2, limites, libellés.
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

    /// Reconnaît les boîtes `[Gmail]/…` et `[Google Mail]/…`.
    fn mailbox_role_hint(&self, mailbox: &RawMailbox) -> Option<MailboxRole> {
        role_for_name(&mailbox.name)
    }

    /// Traduit `X-GM-MSGID`, `X-GM-THRID` et `X-GM-LABELS`.
    fn translate_attributes(&self, raw: &RawMessageAttributes) -> ServerMessageTranslation {
        translate(raw)
    }
}
