#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Le provider de repli accepte tout, devine ses serveurs et n'a aucune particularité.

use mira_provider_generic::GenericProvider;
use mira_vocabulary::{
    AuthMethod, EmailAddress, Provider, ProviderLimits, ProviderQuirks, RawMailbox,
    RawMessageAttributes, Security, ServerMessageTranslation,
};

/// Construit une adresse valide pour un test
fn address(text: &str) -> EmailAddress {
    EmailAddress::parse(text).unwrap()
}

/// Le repli gère toute adresse, y compris celle d'un service connu par ailleurs
#[test]
fn fallback_handles_every_address() {
    for text in ["a@exemple.org", "b@gmail.com", "c@sous.domaine.example"] {
        assert!(GenericProvider.handles_address(&address(text)), "{text}");
    }
}

/// Les serveurs sont devinés d'après le domaine, en TLS implicite
#[test]
fn servers_are_guessed_from_the_domain() {
    let servers = GenericProvider.servers_for(&address("x@Exemple.ORG"));
    assert_eq!(servers.imap.host, "imap.exemple.org");
    assert_eq!(servers.imap.port, 993);
    assert_eq!(servers.imap.security, Security::Tls);
    assert_eq!(servers.smtp.host, "smtp.exemple.org");
    assert_eq!(servers.smtp.port, 465);
    assert_eq!(servers.smtp.security, Security::Tls);
}

/// La fiche du repli : id, aucun domaine, authentifications et limites prudentes
#[test]
fn descriptor_is_the_prudent_fallback() {
    let descriptor = GenericProvider.descriptor();
    assert_eq!(descriptor.id.as_str(), "generic");
    assert!(descriptor.domains.is_empty());
    assert_eq!(
        descriptor.auth_methods,
        vec![AuthMethod::Password, AuthMethod::AppPassword]
    );
    assert_eq!(descriptor.limits, ProviderLimits::default());
    assert_eq!(descriptor.limits.max_connections, 3);
    assert_eq!(descriptor.limits.min_poll_interval_secs, 300);
    assert_eq!(descriptor.quirks, ProviderQuirks::default());
}

/// Aucun indice de rôle et aucune traduction, quels que soient les attributs
#[test]
fn no_role_hint_and_no_translation() {
    let mailbox = RawMailbox {
        name: "Sent".to_string(),
        delimiter: Some("/".to_string()),
        attributes: vec!["\\Sent".to_string()],
    };
    assert_eq!(GenericProvider.mailbox_role_hint(&mailbox), None);
    let raw = RawMessageAttributes {
        items: vec![("X-GM-MSGID".to_string(), "42".to_string())],
    };
    assert_eq!(
        GenericProvider.translate_attributes(&raw),
        ServerMessageTranslation::default()
    );
}
