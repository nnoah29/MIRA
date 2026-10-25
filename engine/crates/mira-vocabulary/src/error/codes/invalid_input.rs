//! Codes de la catégorie `invalid_input`.

use crate::error::code::ErrorCode;

/// Adresse e-mail mal formée.
pub const ADDRESS: ErrorCode = ErrorCode("invalid_input.address");
/// Paramètre obligatoire absent.
pub const MISSING_PARAMETER: ErrorCode = ErrorCode("invalid_input.missing_parameter");
/// Paramètre de valeur invalide.
pub const PARAMETER: ErrorCode = ErrorCode("invalid_input.parameter");
/// Version de contrat demandée incompatible.
pub const CONTRACT_VERSION: ErrorCode = ErrorCode("invalid_input.contract_version");
/// Curseur de pagination invalide.
pub const CURSOR: ErrorCode = ErrorCode("invalid_input.cursor");
/// Clé d'idempotence réutilisée avec une autre commande.
pub const IDEMPOTENCY_CONFLICT: ErrorCode = ErrorCode("invalid_input.idempotency_conflict");
/// Nom de modèle de message vide.
pub const TEMPLATE_NAME_EMPTY: ErrorCode = ErrorCode("invalid_input.template_name_empty");
/// Nom de modèle de message déjà pris.
pub const TEMPLATE_NAME_TAKEN: ErrorCode = ErrorCode("invalid_input.template_name_taken");
/// Nom, objet ou corps de modèle de message trop long.
pub const TEMPLATE_TOO_LONG: ErrorCode = ErrorCode("invalid_input.template_too_long");
