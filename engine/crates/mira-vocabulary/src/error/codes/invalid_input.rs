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
/// Fichier d'export dont la syntaxe YAML est invalide.
pub const YAML_SYNTAX: ErrorCode = ErrorCode("invalid_input.yaml_syntax");
/// Fichier d'export d'une version du format inconnue.
pub const EXPORT_VERSION_UNSUPPORTED: ErrorCode =
    ErrorCode("invalid_input.export_version_unsupported");
/// Fichier d'export d'un autre type que celui attendu.
pub const EXPORT_KIND_MISMATCH: ErrorCode = ErrorCode("invalid_input.export_kind_mismatch");
/// Fichier d'export trop gros (taille ou nombre d'éléments).
pub const EXPORT_TOO_LARGE: ErrorCode = ErrorCode("invalid_input.export_too_large");
/// Règle d'un fichier d'import dont l'identifiant figure déjà plus haut dans le même fichier.
pub const RULE_DUPLICATE_IN_FILE: ErrorCode = ErrorCode("invalid_input.rule_duplicate_in_file");
/// Texte trop long pour le service d'IA.
pub const AI_INPUT_TOO_LARGE: ErrorCode = ErrorCode("invalid_input.ai_input_too_large");
/// Adresse de service d'IA invalide.
pub const AI_URL_INVALID: ErrorCode = ErrorCode("invalid_input.ai_url_invalid");
/// Clé de service d'IA vide ou mal formée.
pub const AI_KEY_INVALID: ErrorCode = ErrorCode("invalid_input.ai_key_invalid");
