//! Listes de messages par vue : une boîte, ou une vue standard sur plusieurs comptes (7.10).

use mira_vocabulary::{AccountId, MessageSummary, MiraError, Page, PageRequest};
use mira_vocabulary::{StandardView, ViewRef};

use super::builder::QueryBuilder;
use super::clauses::apply_filters;
use super::filters::MessageFilters;
use crate::storage::Tx;

const VISIBLE: &str =
    "EXISTS (SELECT 1 FROM appearances a WHERE a.message_id = m.id AND a.deleted = 0)";
const IN_INBOX: &str = "EXISTS (SELECT 1 FROM appearances a JOIN mailboxes b ON b.id = a.mailbox_id WHERE a.message_id = m.id AND a.deleted = 0 AND b.role = 'inbox')";

impl Tx<'_> {
    /// Liste une page de messages d'une vue, filtrés, du plus récent au plus ancien
    pub fn list_view_messages(
        &self,
        accounts: &[AccountId],
        view: &ViewRef,
        filters: &MessageFilters,
        request: &PageRequest,
    ) -> Result<Page<MessageSummary>, MiraError> {
        let mut builder = match view {
            ViewRef::Mailbox { mailbox } => self.mailbox_builder(mailbox)?,
            ViewRef::Standard { view } => standard_builder(accounts, *view),
        };
        apply_filters(&mut builder, filters);
        self.run_message_page(builder, request)
    }
}

/// Requête d'une vue standard sur les comptes donnés ; le `+` écarte l'index par compte, qui obligerait à trier
fn standard_builder(accounts: &[AccountId], view: StandardView) -> QueryBuilder {
    let mut builder = QueryBuilder::new("messages m");
    let names: Vec<&str> = accounts.iter().map(AccountId::as_str).collect();
    let list = builder.bind_list(&names);
    builder.require(format!("+m.account_id IN {list}"));
    match view {
        StandardView::Unified => builder.require(IN_INBOX),
        StandardView::Unread => {
            builder.require("m.seen = 0");
            builder.require(VISIBLE);
        }
        StandardView::Flagged => {
            builder.require("m.flagged = 1");
            builder.require(VISIBLE);
        }
    }
    builder
}
