//! Catalogue des codes d'erreur stables, un fichier par catégorie.

use super::code::ErrorCode;

pub mod authentication;
pub mod cancelled;
pub mod incompatible_state;
pub mod internal;
pub mod invalid_input;
pub mod local_storage;
pub mod network;
pub mod not_found;
pub mod secrets;
pub mod server;
pub mod tls;

/// Tous les codes du catalogue (sert au test d'unicité et de préfixe).
pub const ALL: &[ErrorCode] = &[
    invalid_input::ADDRESS,
    invalid_input::MISSING_PARAMETER,
    invalid_input::PARAMETER,
    invalid_input::CONTRACT_VERSION,
    invalid_input::CURSOR,
    invalid_input::IDEMPOTENCY_CONFLICT,
    invalid_input::TEMPLATE_NAME_EMPTY,
    invalid_input::TEMPLATE_NAME_TAKEN,
    invalid_input::TEMPLATE_TOO_LONG,
    invalid_input::AI_INPUT_TOO_LARGE,
    invalid_input::AI_URL_INVALID,
    invalid_input::AI_KEY_INVALID,
    invalid_input::AI_JOURNAL_LIMIT,
    not_found::ACCOUNT,
    not_found::MESSAGE,
    not_found::MAILBOX,
    not_found::IDENTITY,
    not_found::DRAFT,
    not_found::ACTION,
    not_found::OPERATION,
    not_found::PROVIDER,
    not_found::ATTACHMENT,
    not_found::THREAD,
    not_found::RULE,
    not_found::LOCAL_LABEL,
    not_found::LABEL,
    not_found::TEMPLATE,
    not_found::AI_MODEL,
    not_found::AI_CONNECTION,
    not_found::AI_PROVIDER,
    incompatible_state::SYSTEM_MAILBOX,
    incompatible_state::ACCOUNT_DELETING,
    incompatible_state::PROFILE_LOCKED,
    incompatible_state::MULTIPLE_LABELS_UNSUPPORTED,
    incompatible_state::BODY_UNAVAILABLE_OFFLINE,
    incompatible_state::ACTION_NOT_CANCELLABLE,
    incompatible_state::ACCOUNT_PAUSED,
    incompatible_state::DRAFT_NOT_EDITABLE,
    incompatible_state::ACTION_NOT_RETRYABLE,
    incompatible_state::SYNC_DISABLED,
    incompatible_state::MAILBOX_CHANGED,
    incompatible_state::AI_CONSENT_REQUIRED,
    incompatible_state::AI_KEY_MISSING,
    incompatible_state::AI_MODEL_MISSING,
    incompatible_state::AI_CONTENT_NOT_ALLOWED,
    network::OFFLINE,
    network::TIMEOUT,
    network::UNREACHABLE,
    network::CONNECTION_LOST,
    tls::INVALID_CERTIFICATE,
    tls::HANDSHAKE,
    tls::PLAINTEXT_REFUSED,
    authentication::REJECTED,
    authentication::TOKEN_EXPIRED,
    authentication::TOKEN_REVOKED,
    authentication::APP_PASSWORD_REQUIRED,
    authentication::OAUTH_CANCELLED,
    authentication::OAUTH_STATE_MISMATCH,
    authentication::AI_KEY_REJECTED,
    server::QUOTA_EXCEEDED,
    server::MESSAGE_TOO_LARGE,
    server::RATE_LIMITED,
    server::MAILBOX_MISSING,
    server::EXTENSION_MISSING,
    server::REJECTED,
    server::PROTOCOL,
    server::AI_RATE_LIMITED,
    server::AI_UNAVAILABLE,
    server::AI_BAD_RESPONSE,
    local_storage::DISK_FULL,
    local_storage::CORRUPTED,
    local_storage::MIGRATION_REQUIRED,
    local_storage::IO,
    secrets::KEYRING_UNAVAILABLE,
    secrets::KEYRING_LOCKED,
    secrets::SECRET_MISSING,
    cancelled::BY_USER,
    internal::BUG,
];
