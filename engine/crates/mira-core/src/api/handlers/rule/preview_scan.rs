//! Parcours borné de l'aperçu : les messages les plus récents de la portée, évalués un à un.

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

/// Messages candidats des comptes, du plus récent au plus ancien, au plus `PREVIEW_SCAN_LIMIT`
fn recent_candidates(
    tx: &Tx<'_>,
    accounts: &[AccountId],
) -> Result<Vec<MessageSummary>, MiraError> {
    let mut all = Vec::new();
    for account in accounts {
        all.extend(tx.rule_candidates(account, PREVIEW_SCAN_LIMIT)?);
    }
    all.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.id.cmp(&a.1.id)));
    all.truncate(PREVIEW_SCAN_LIMIT as usize);
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

/// Compte les messages touchés par la règle et garde au plus `samples` exemples, les plus récents
pub(super) fn scan_candidates(
    tx: &Tx<'_>,
    rule: &Rule,
    now: Timestamp,
    samples: u32,
) -> Result<RulePreview, MiraError> {
    let candidates = recent_candidates(tx, &scope_accounts(tx, &rule.scope)?)?;
    let scanned = u64::try_from(candidates.len()).unwrap_or(u64::MAX);
    let mut preview = RulePreview {
        matched: 0,
        scanned,
        samples: Vec::new(),
    };
    let mut cache = HashMap::new();
    for summary in candidates {
        let own = cached_own_addresses(tx, &mut cache, &summary.account)?;
        let id = summary.id.clone();
        let context = message_context(tx, summary, own, now)?;
        if !rule_fires_on(rule, &context) {
            continue;
        }
        preview.matched += 1;
        if preview.samples.len() < samples as usize {
            preview.samples.push(id);
        }
    }
    Ok(preview)
}
