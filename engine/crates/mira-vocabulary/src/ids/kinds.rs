//! Marqueurs de type et alias des identifiants du domaine.

use super::Id;

/// Marqueur des identifiants de profil.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProfileMarker;

/// Marqueur des identifiants de compte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AccountMarker;

/// Marqueur des identifiants d'identité d'envoi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IdentityMarker;

/// Marqueur des identifiants de boîte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MailboxMarker;

/// Marqueur des identifiants de message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MessageMarker;

/// Marqueur des identifiants de pièce jointe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AttachmentMarker;

/// Marqueur des identifiants de fil.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ThreadMarker;

/// Marqueur des identifiants de brouillon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DraftMarker;

/// Marqueur des identifiants d'action en file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ActionMarker;

/// Marqueur des identifiants d'opération longue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OperationMarker;

/// Marqueur des identifiants de vue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ViewMarker;

/// Marqueur des identifiants de règle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuleMarker;

/// Identifiant de règle.
pub type RuleId = Id<RuleMarker>;

/// Marqueur des identifiants de libellé local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalLabelMarker;

/// Identifiant de libellé local.
pub type LocalLabelId = Id<LocalLabelMarker>;
/// Identifiant de profil.
pub type ProfileId = Id<ProfileMarker>;
/// Identifiant de compte.
pub type AccountId = Id<AccountMarker>;
/// Identifiant d'identité d'envoi.
pub type IdentityId = Id<IdentityMarker>;
/// Identifiant de boîte.
pub type MailboxId = Id<MailboxMarker>;
/// Identifiant de message.
pub type MessageId = Id<MessageMarker>;
/// Identifiant de pièce jointe.
pub type AttachmentId = Id<AttachmentMarker>;
/// Identifiant de fil.
pub type ThreadId = Id<ThreadMarker>;
/// Identifiant de brouillon.
pub type DraftId = Id<DraftMarker>;
/// Identifiant d'action en file.
pub type ActionId = Id<ActionMarker>;
/// Identifiant d'opération longue.
pub type OperationId = Id<OperationMarker>;
/// Identifiant de vue.
pub type ViewId = Id<ViewMarker>;
