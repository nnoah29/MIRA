//! Règles de test et relevés de la file d'actions et des événements de règles.

use mira_vocabulary::{
    ActionKind, Condition, ConditionGroup, ConditionTest, EventKind, PendingAction, Rule,
    RuleAction, RuleId, RuleScope, TextMatcher, TextOp,
};

use super::mirror::account;
use super::waiting::count_events;
use super::world::SyncWorld;

/// Règle sur tous les comptes qui vise tout message (objet qui ne contient pas un texte introuvable)
pub fn catch_all(id: &str, actions: Vec<RuleAction>) -> Rule {
    let never = TextMatcher {
        op: TextOp::Contains,
        values: vec!["zzz-introuvable".to_string()],
    };
    Rule {
        id: RuleId::from_raw(id),
        name: id.to_string(),
        enabled: true,
        scope: RuleScope::AllAccounts,
        alternatives: vec![ConditionGroup {
            conditions: vec![Condition {
                negated: true,
                test: ConditionTest::Subject { matcher: never },
            }],
        }],
        actions,
        schema_version: 1,
    }
}

/// Enregistre une règle dans la base du monde
pub fn store_rule(world: &SyncWorld, rule: &Rule) {
    world.db.write(|tx| tx.insert_rule(rule)).unwrap();
}

/// Actions en file du compte de test
pub fn queued(world: &SyncWorld) -> Vec<PendingAction> {
    world
        .db
        .read(|tx| tx.list_actions(&account(), &[]))
        .unwrap()
}

/// Nombre d'actions « marquer lu » en file
pub fn mark_read_count(world: &SyncWorld) -> usize {
    let is_read = |a: &&PendingAction| matches!(a.kind, ActionKind::MarkRead { read: true });
    queued(world).iter().filter(is_read).count()
}

/// Nombre d'événements « règle appliquée »
pub fn rule_applied(world: &SyncWorld) -> usize {
    count_events(&world.recording, |k| {
        matches!(k, EventKind::RuleApplied { .. })
    })
}

/// Valeurs `silenced` des événements « nouveau message », dans l'ordre
pub fn silenced_flags(world: &SyncWorld) -> Vec<bool> {
    let flag = |kind: &EventKind| match kind {
        EventKind::NewMessageReceived { silenced, .. } => Some(*silenced),
        _ => None,
    };
    world.recording.kinds().iter().filter_map(flag).collect()
}
