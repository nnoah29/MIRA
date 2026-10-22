//! Condition inversable et groupe de conditions reliées par ET.

use serde::{Deserialize, Serialize};

use super::test::ConditionTest;

/// Test élémentaire, éventuellement inversé.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Condition {
    pub negated: bool,
    pub test: ConditionTest,
}

/// Suite de conditions reliées par ET ; les groupes d'une règle sont reliés par OU.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConditionGroup {
    pub conditions: Vec<Condition>,
}
