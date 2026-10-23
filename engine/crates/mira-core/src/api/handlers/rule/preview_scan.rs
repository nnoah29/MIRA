//! Parcours borné de l'aperçu : contextes lus sous le verrou de la base, évaluation faite ensuite, hors verrou.

use std::collections::HashMap;

use mira_vocabulary::{
    AccountId, EmailAddress, MessageSummary, MiraError, Rule, RuleScope, Timestamp,
};

use crate::api::contract::RulePreview;
use crate::rule_context::{message_context, own_addresses};
use crate::rules::context::MessageContext;
use crate::rules::evaluate::evaluate;
use crate::storage::Tx;

/// Nombre maximal de messages examinés par un aperçu, tous comptes confondus (les plus récents).
pub const PREVIEW_SCAN_LIMIT: u32 = 1000;

/// Comptes couverts par la portée, chacun une seule fois ; un compte inconnu ne fournit aucun message
fn scope_accounts(tx: &Tx<'_>, scope: &RuleScope) -> Result<Vec<AccountId>, MiraError> {
    let mut accounts = match scope {
        RuleScope::AllAccounts => tx.list_accounts()?.into_iter().map(|a| a.id).collect(),
        RuleScope::Accounts(accounts) => accounts.clone(),
    };
    accounts.sort();
    accounts.dedup();
    Ok(accounts)
}

/// Messages candidats des comptes, du plus récent au plus ancien, au plus `limit`
fn recent_candidates(
    tx: &Tx<'_>,
    accounts: &[AccountId],
    limit: u32,
) -> Result<Vec<MessageSummary>, MiraError> {
    let mut all = Vec::new();
    for account in accounts {
        all.extend(tx.rule_candidates(account, limit)?);
    }
    all.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.id.cmp(&a.1.id)));
    all.truncate(limit as usize);
    Ok(all.into_iter().map(|(_, summary)| summary).collect())
}

/// Adresses « moi » du compte, lues une seule fois par compte
fn cached_own_addresses(
    tx: &Tx<'_>,
    cache: &mut HashMap<AccountId, Vec<EmailAddress>>,
    account: &AccountId,
) -> Result<Vec<EmailAddress>, MiraError> {
    if let Some(found) = cache.get(account) {
        return Ok(found.clone());
    }
    let addresses = own_addresses(tx, account)?;
    cache.insert(account.clone(), addresses.clone());
    Ok(addresses)
}

/// Vrai si l'évaluateur déclenche la règle seule sur ce message
fn rule_fires_on(rule: &Rule, context: &MessageContext) -> bool {
    let outcome = evaluate(std::slice::from_ref(rule), context);
    !outcome.matched.is_empty()
}

/// Lit, sous le verrou de la base, le contexte des messages candidats de la portée (au plus `limit`)
pub(super) fn load_contexts(
    tx: &Tx<'_>,
    scope: &RuleScope,
    now: Timestamp,
    limit: u32,
) -> Result<Vec<MessageContext>, MiraError> {
    let candidates = recent_candidates(tx, &scope_accounts(tx, scope)?, limit)?;
    let mut cache = HashMap::new();
    let mut contexts = Vec::with_capacity(candidates.len());
    for summary in candidates {
        let own = cached_own_addresses(tx, &mut cache, &summary.account)?;
        contexts.push(message_context(tx, summary, own, now)?);
    }
    Ok(contexts)
}

/// Évalue la règle sans accès à la base : messages touchés et au plus `samples` exemples, les plus récents
pub(super) fn count_matches(
    rule: &Rule,
    contexts: Vec<MessageContext>,
    samples: u32,
) -> RulePreview {
    let mut preview = RulePreview {
        matched: 0,
        scanned: u64::try_from(contexts.len()).unwrap_or(u64::MAX),
        samples: Vec::new(),
    };
    for context in contexts {
        if !rule_fires_on(rule, &context) {
            continue;
        }
        preview.matched += 1;
        if preview.samples.len() < samples as usize {
            preview.samples.push(context.summary.id);
        }
    }
    preview
}
