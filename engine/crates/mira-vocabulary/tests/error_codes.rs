#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::HashSet;

use mira_vocabulary::error::codes::{self, network};
use mira_vocabulary::{ErrorCategory, ErrorContext, MiraError, SuggestedAction};

const CATEGORIES: [ErrorCategory; 11] = [
    ErrorCategory::InvalidInput,
    ErrorCategory::NotFound,
    ErrorCategory::IncompatibleState,
    ErrorCategory::Network,
    ErrorCategory::Tls,
    ErrorCategory::Authentication,
    ErrorCategory::Server,
    ErrorCategory::LocalStorage,
    ErrorCategory::Secrets,
    ErrorCategory::Cancelled,
    ErrorCategory::Internal,
];

/// Invariant 31 : les codes sont uniques et préfixés par le nom d'une catégorie.
#[test]
fn invariant_31_error_codes_are_unique_and_prefixed_by_category() {
    let mut seen = HashSet::new();
    for code in codes::ALL {
        assert!(seen.insert(code.0), "code en double : {}", code.0);
        let (prefix, subject) = code.0.split_once('.').unwrap();
        assert!(!subject.is_empty(), "sujet vide : {}", code.0);
        assert!(
            CATEGORIES.iter().any(|c| c.as_str() == prefix),
            "préfixe inconnu : {}",
            code.0
        );
    }
}

/// Chaque catégorie possède au moins un code dans le catalogue.
#[test]
fn every_category_has_at_least_one_code() {
    for category in CATEGORIES {
        let prefix = format!("{}.", category.as_str());
        assert!(
            codes::ALL.iter().any(|c| c.0.starts_with(&prefix)),
            "{prefix}"
        );
    }
}

/// Une erreur sérialisée expose catégorie, code, message, rejouable, action et contexte.
#[test]
fn serialized_error_contains_contract_fields() {
    let error = MiraError::new(ErrorCategory::Network, network::TIMEOUT, "délai dépassé")
        .with_action(SuggestedAction::RetryLater);
    let json = serde_json::to_value(&error).unwrap();
    assert_eq!(json["category"], "network");
    assert_eq!(json["code"], "network.timeout");
    assert_eq!(json["retryable"], true);
    assert_eq!(json["suggested_action"], "retry_later");
    assert!(json["context"].is_object());
    let back: MiraError = serde_json::from_value(json).unwrap();
    assert_eq!(back, error);
}

/// Seules les erreurs réseau sont rejouables par défaut, et `retryable` peut corriger cela.
#[test]
fn retryable_defaults_to_network_only_and_can_be_overridden() {
    let net = MiraError::new(ErrorCategory::Network, network::OFFLINE, "hors ligne");
    let bug = MiraError::new(ErrorCategory::Internal, codes::internal::BUG, "défaut");
    assert!(net.retryable);
    assert!(!bug.retryable);
    assert!(bug.clone().retryable(true).retryable);
    assert_eq!(net.to_string(), "network.offline: hors ligne");
    assert_eq!(*bug.context, ErrorContext::default());
}
