//! Interface qu'implémente chaque provider (D-05, D-07, D-08) : pure, sans entrée-sortie.

use std::fmt::Debug;

use super::descriptor::ProviderDescriptor;
use super::raw::{RawMailbox, RawMessageAttributes, ServerMessageTranslation};
use crate::address::EmailAddress;
use crate::domain::mailbox::MailboxRole;
use crate::domain::servers::ServerSettings;

/// Un service de messagerie connu du moteur ; seul `mira-assembly` les connaît tous.
pub trait Provider: Debug + Send + Sync {
    /// Décrit le provider (données pures).
    fn descriptor(&self) -> ProviderDescriptor;

    /// Indique si le provider gère cette adresse, d'après son domaine.
    fn handles_address(&self, address: &EmailAddress) -> bool;

    /// Donne les serveurs à utiliser pour cette adresse.
    fn servers_for(&self, address: &EmailAddress) -> ServerSettings;

    /// Propose un rôle pour une boîte quand le service s'écarte de la norme.
    fn mailbox_role_hint(&self, mailbox: &RawMailbox) -> Option<MailboxRole>;

    /// Traduit les attributs bruts propres au service en notions du domaine.
    fn translate_attributes(&self, raw: &RawMessageAttributes) -> ServerMessageTranslation;
}
