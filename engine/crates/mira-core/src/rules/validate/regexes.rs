//! Contrôle des expressions régulières avec la même compilation que l'évaluateur.

use mira_vocabulary::{MiraError, Rule, TextOp};

use super::limits::MAX_REGEX_LEN;
use super::matchers::rule_matchers;
use crate::actions::errors::input::invalid_parameter;
use crate::rules::text_match::compile_regex;

/// Refuse une regex trop longue ou qui ne compile pas dans la limite de taille de l'évaluateur.
pub(super) fn check_regexes(rule: &Rule) -> Result<(), MiraError> {
    let patterns = rule_matchers(rule)
        .into_iter()
        .filter(|matcher| matcher.op == TextOp::Regex)
        .flat_map(|matcher| matcher.values.iter());
    for pattern in patterns {
        if pattern.len() > MAX_REGEX_LEN {
            return Err(invalid_parameter(
                "une expression régulière est trop longue",
            ));
        }
        if compile_regex(pattern).is_err() {
            return Err(invalid_parameter("une expression régulière est invalide"));
        }
    }
    Ok(())
}
