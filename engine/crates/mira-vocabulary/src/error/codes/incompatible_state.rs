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
/// Le brouillon n'est plus en cours d'édition (planifié ou en envoi).
pub const DRAFT_NOT_EDITABLE: ErrorCode = ErrorCode("incompatible_state.draft_not_editable");
/// Seule une action échouée peut être relancée.
pub const ACTION_NOT_RETRYABLE: ErrorCode = ErrorCode("incompatible_state.action_not_retryable");
/// Le serveur a renuméroté la boîte depuis le dernier relevé : il faut la resynchroniser.
pub const MAILBOX_CHANGED: ErrorCode = ErrorCode("incompatible_state.mailbox_changed");
/// La synchronisation en arrière-plan n'est pas active dans ce moteur.
pub const SYNC_DISABLED: ErrorCode = ErrorCode("incompatible_state.sync_disabled");
/// L'utilisateur n'a pas encore consenti à l'envoi de texte à ce service d'IA.
pub const AI_CONSENT_REQUIRED: ErrorCode = ErrorCode("incompatible_state.ai_consent_required");
/// La connexion d'IA n'a pas de clé enregistrée.
pub const AI_KEY_MISSING: ErrorCode = ErrorCode("incompatible_state.ai_key_missing");
/// Aucun modèle d'IA n'est choisi pour cette connexion.
pub const AI_MODEL_MISSING: ErrorCode = ErrorCode("incompatible_state.ai_model_missing");
/// Le type de contenu à envoyer n'est pas autorisé pour cette connexion d'IA.
pub const AI_CONTENT_NOT_ALLOWED: ErrorCode =
    ErrorCode("incompatible_state.ai_content_not_allowed");
