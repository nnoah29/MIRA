//! Codes de la catégorie `incompatible_state`.

use crate::error::code::ErrorCode;

/// Opération interdite sur une boîte système.
pub const SYSTEM_MAILBOX: ErrorCode = ErrorCode("incompatible_state.system_mailbox");
/// Le compte est en cours de suppression.
pub const ACCOUNT_DELETING: ErrorCode = ErrorCode("incompatible_state.account_deleting");
/// Le profil est déjà utilisé par un autre moteur.
pub const PROFILE_LOCKED: ErrorCode = ErrorCode("incompatible_state.profile_locked");
/// Second libellé refusé sur un compte à dossiers (D-17).
pub const MULTIPLE_LABELS_UNSUPPORTED: ErrorCode =
    ErrorCode("incompatible_state.multiple_labels_unsupported");
/// Corps du message absent hors ligne.
pub const BODY_UNAVAILABLE_OFFLINE: ErrorCode =
    ErrorCode("incompatible_state.body_unavailable_offline");
/// L'action n'est plus annulable.
pub const ACTION_NOT_CANCELLABLE: ErrorCode =
    ErrorCode("incompatible_state.action_not_cancellable");
/// Le compte est en pause.
pub const ACCOUNT_PAUSED: ErrorCode = ErrorCode("incompatible_state.account_paused");
