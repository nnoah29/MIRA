#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_vocabulary::{
    AuthMethod, EmailAddress, MailboxRole, Provider, ProviderDescriptor, ProviderId,
    ProviderLimits, ProviderQuirks, RawMailbox, RawMessageAttributes, Security, ServerEndpoint,
    ServerMessageTranslation, ServerSettings,
};

#[derive(Debug)]
struct FakeProvider;

impl Provider for FakeProvider {
    /// Décrit un provider factice qui gère `fake.example`.
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: ProviderId::new("fake"),
            display_name: "Fake".to_string(),
            domains: vec!["fake.example".to_string()],
            auth_methods: vec![AuthMethod::AppPassword],
            limits: ProviderLimits::default(),
            quirks: ProviderQuirks::default(),
        }
    }

    /// Gère les adresses du domaine `fake.example`.
    fn handles_address(&self, address: &EmailAddress) -> bool {
        address.domain() == "fake.example"
    }

    /// Rend toujours les mêmes serveurs TLS.
    fn servers_for(&self, address: &EmailAddress) -> ServerSettings {
        let endpoint = |prefix: &str, port| ServerEndpoint {
            host: format!("{prefix}.{}", address.domain()),
            port,
            security: Security::Tls,
        };
        ServerSettings {
            imap: endpoint("imap", 993),
            smtp: endpoint("smtp", 465),
        }
    }

    /// Reconnaît l'attribut `\Sent` comme boîte des envoyés.
    fn mailbox_role_hint(&self, mailbox: &RawMailbox) -> Option<MailboxRole> {
        mailbox
            .attributes
            .iter()
            .any(|a| a == "\\Sent")
            .then_some(MailboxRole::Sent)
    }

    /// Lit l'identifiant stable dans `X-GM-MSGID`.
    fn translate_attributes(&self, raw: &RawMessageAttributes) -> ServerMessageTranslation {
        let stable_id = raw
            .items
            .iter()
            .find(|(name, _)| name == "X-GM-MSGID")
            .map(|(_, v)| v.clone());
        ServerMessageTranslation {
            stable_id,
            ..ServerMessageTranslation::default()
        }
    }
}

/// Le trait s'utilise comme objet partagé et ses cinq méthodes répondent comme prévu.
#[test]
fn provider_trait_works_through_a_trait_object() {
    let provider: Box<dyn Provider> = Box::new(FakeProvider);
    let address = EmailAddress::parse("bob@Fake.example").unwrap();
    assert!(provider.handles_address(&address));
    assert_eq!(provider.descriptor().id.as_str(), "fake");
    assert_eq!(
        provider.servers_for(&address).imap.host,
        "imap.fake.example"
    );
    let mailbox = RawMailbox {
        name: "Sent".to_string(),
        delimiter: Some("/".to_string()),
        attributes: vec!["\\Sent".to_string()],
    };
    assert_eq!(
        provider.mailbox_role_hint(&mailbox),
        Some(MailboxRole::Sent)
    );
    let raw = RawMessageAttributes {
        items: vec![("X-GM-MSGID".to_string(), "42".to_string())],
    };
    assert_eq!(
        provider.translate_attributes(&raw).stable_id.as_deref(),
        Some("42")
    );
}
