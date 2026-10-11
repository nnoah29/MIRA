//! Faux provider à dossiers : identifiant et domaine au choix du test.

use mira_vocabulary::{
    AuthMethod, EmailAddress, MailboxRole, Provider, ProviderDescriptor, ProviderId,
    ProviderLimits, ProviderQuirks, RawMailbox, RawMessageAttributes, Security, ServerEndpoint,
    ServerMessageTranslation, ServerSettings,
};

/// Faux provider à dossiers pour les tests du cœur.
#[derive(Debug, Clone)]
pub struct FakeFoldersProvider {
    id: String,
    domain: String,
}

impl FakeFoldersProvider {
    /// Crée un faux provider d'identifiant et de domaine donnés (domaine vide : n'en gère aucun).
    pub fn new(id: &str, domain: &str) -> Self {
        Self {
            id: id.to_string(),
            domain: domain.to_string(),
        }
    }
}

impl Provider for FakeFoldersProvider {
    /// Décrit un service à dossiers sur le domaine choisi.
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: ProviderId::new(&self.id),
            display_name: "Fake folders".to_string(),
            domains: vec![self.domain.clone()],
            auth_methods: vec![AuthMethod::Password],
            limits: ProviderLimits::default(),
            quirks: ProviderQuirks::default(),
        }
    }

    /// Gère exactement le domaine choisi.
    fn handles_address(&self, address: &EmailAddress) -> bool {
        address.domain() == self.domain
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

    /// Prend la boîte `Corbeille` pour du courrier indésirable, au contraire de la règle générale.
    fn mailbox_role_hint(&self, mailbox: &RawMailbox) -> Option<MailboxRole> {
        (mailbox.name == "Corbeille").then_some(MailboxRole::Junk)
    }

    /// Ne traduit rien.
    fn translate_attributes(&self, _raw: &RawMessageAttributes) -> ServerMessageTranslation {
        ServerMessageTranslation::default()
    }
}
