//! Exécution d'une requête de liste : curseur de reprise, limite bornée, découpe de la page.

use mira_vocabulary::{MessageSummary, MiraError, Page, PageRequest};
use rusqlite::params_from_iter;

use super::builder::QueryBuilder;
use crate::storage::Tx;
use crate::storage::pagination::{cursor_bounds, finish_page};
use crate::storage::repositories::messages::MESSAGE_COLUMNS;

impl Tx<'_> {
    /// Ajoute la reprise après le curseur, exécute la requête et coupe la page (invariant 29)
    pub(super) fn run_message_page(
        &self,
        mut builder: QueryBuilder,
        request: &PageRequest,
    ) -> Result<Page<MessageSummary>, MiraError> {
        let limit = request.bounded_limit();
        if let (Some(key), Some(id)) = cursor_bounds(request)? {
            let (key, id) = (builder.bind(key), builder.bind(id));
            builder.require(format!("(m.sort_date, m.id) < ({key}, {id})"));
        }
        let (sql, values) = builder.finish(MESSAGE_COLUMNS, i64::from(limit) + 1);
        let rows = self.query_summaries(&sql, params_from_iter(values))?;
        Ok(finish_page(rows, limit))
    }
}
