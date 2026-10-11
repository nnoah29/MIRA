//! Faux provider à libellés : gère le domaine `labels.test`.

use mira_vocabulary::{
    AuthMethod, EmailAddress, MailboxModel, MailboxRole, Provider, ProviderDescriptor, ProviderId,
    ProviderLimits, ProviderQuirks, RawMailbox, RawMessageAttributes, Security, ServerEndpoint,
    ServerMessageTranslation, ServerSettings,
};

/// Faux provider à libellés pour les tests du cœur.
#[derive(Debug, Clone, Copy, Default)]
pub struct FakeLabelsProvider;

impl Provider for FakeLabelsProvider {
    /// Décrit un service à libellés sur `labels.test`.
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: ProviderId::new("fake_labels"),
            display_name: "Fake labels".to_string(),
            domains: vec!["labels.test".to_string()],
            auth_methods: vec![AuthMethod::AppPassword],
            limits: ProviderLimits::default(),
            quirks: ProviderQuirks {
                mailbox_model: MailboxModel::Labels,
                ..ProviderQuirks::default()
            },
        }
    }

    /// Gère exactement le domaine `labels.test`.
    fn handles_address(&self, address: &EmailAddress) -> bool {
        address.domain() == "labels.test"
    }

    /// Donne des serveurs TLS fictifs.
    fn servers_for(&self, address: &EmailAddress) -> ServerSettings {
        let endpoint = |prefix: &str, port: u16| ServerEndpoint {
            host: format!("{prefix}.{}", address.domain()),
            port,
            security: Security::Tls,
        };
        ServerSettings {
            imap: endpoint("imap", 993),
            smtp: endpoint("smtp", 465),
        }
    }

    /// Reconnaît la boîte `Tout le courrier` comme boîte « tous les messages ».
    fn mailbox_role_hint(&self, mailbox: &RawMailbox) -> Option<MailboxRole> {
        (mailbox.name == "Tout le courrier").then_some(MailboxRole::All)
    }

    /// Ne traduit rien.
    fn translate_attributes(&self, _raw: &RawMessageAttributes) -> ServerMessageTranslation {
        ServerMessageTranslation::default()
    }
}
