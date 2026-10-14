//! Faux provider des tests de synchronisation : identifiant stable optionnel, limites réglables.

use mira_vocabulary::{
    AuthMethod, EmailAddress, MailboxModel, MailboxRole, Provider, ProviderDescriptor, ProviderId,
    ProviderLimits, ProviderQuirks, RawMailbox, RawMessageAttributes, Security, ServerEndpoint,
    ServerMessageTranslation, ServerSettings,
};

use super::provider_builders::raw_item;

/// Provider de test : dossiers ou libellés, avec ou sans identifiant stable, limites au choix.
#[derive(Debug, Clone)]
pub struct SyncTestProvider {
    pub stable_ids: bool,
    pub limits: ProviderLimits,
}

impl Provider for SyncTestProvider {
    /// Décrit le service de test
    fn descriptor(&self) -> ProviderDescriptor {
        let extra: &[&str] = if self.stable_ids {
            &["X-GM-MSGID", "X-GM-THRID"]
        } else {
            &[]
        };
        ProviderDescriptor {
            id: ProviderId::new("fake_sync"),
            display_name: "Fake sync".to_string(),
            domains: vec!["example.org".to_string()],
            auth_methods: vec![AuthMethod::AppPassword],
            limits: self.limits,
            quirks: ProviderQuirks {
                mailbox_model: MailboxModel::Folders,
                extra_fetch_attributes: extra.iter().map(ToString::to_string).collect(),
                ..ProviderQuirks::default()
            },
        }
    }

    /// Gère le domaine example.org
    fn handles_address(&self, address: &EmailAddress) -> bool {
        address.domain() == "example.org"
    }

    /// Donne des serveurs fictifs
    fn servers_for(&self, address: &EmailAddress) -> ServerSettings {
        let endpoint = |port: u16| ServerEndpoint {
            host: format!("mail.{}", address.domain()),
            port,
            security: Security::Tls,
        };
        ServerSettings {
            imap: endpoint(993),
            smtp: endpoint(465),
        }
    }

    /// Aucun indice : la règle générale s'applique
    fn mailbox_role_hint(&self, _mailbox: &RawMailbox) -> Option<MailboxRole> {
        None
    }

    /// Lit l'identifiant stable et l'indice de fil dans les attributs du serveur
    fn translate_attributes(&self, raw: &RawMessageAttributes) -> ServerMessageTranslation {
        ServerMessageTranslation {
            stable_id: raw_item(raw, "X-GM-MSGID"),
            labels: Vec::new(),
            thread_hint: raw_item(raw, "X-GM-THRID"),
        }
    }
}
