//! Marqueurs de type et alias des identifiants du domaine.

use super::Id;

/// Marqueur des identifiants de profil.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProfileKind;

/// Marqueur des identifiants de compte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AccountKind;

/// Marqueur des identifiants d'identité d'envoi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IdentityKind;

/// Marqueur des identifiants de boîte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MailboxKind;

/// Marqueur des identifiants de message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MessageKind;

/// Marqueur des identifiants de pièce jointe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AttachmentKind;

/// Marqueur des identifiants de fil.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ThreadKind;

/// Marqueur des identifiants de brouillon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DraftKind;

/// Marqueur des identifiants d'action en file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ActionKind;

/// Marqueur des identifiants d'opération longue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OperationKind;

/// Marqueur des identifiants de vue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ViewKind;

/// Identifiant de profil.
pub type ProfileId = Id<ProfileKind>;
/// Identifiant de compte.
pub type AccountId = Id<AccountKind>;
/// Identifiant d'identité d'envoi.
pub type IdentityId = Id<IdentityKind>;
/// Identifiant de boîte.
pub type MailboxId = Id<MailboxKind>;
/// Identifiant de message.
pub type MessageId = Id<MessageKind>;
/// Identifiant de pièce jointe.
pub type AttachmentId = Id<AttachmentKind>;
/// Identifiant de fil.
pub type ThreadId = Id<ThreadKind>;
/// Identifiant de brouillon.
pub type DraftId = Id<DraftKind>;
/// Identifiant d'action en file.
pub type ActionId = Id<ActionKind>;
/// Identifiant d'opération longue.
pub type OperationId = Id<OperationKind>;
/// Identifiant de vue.
pub type ViewId = Id<ViewKind>;
