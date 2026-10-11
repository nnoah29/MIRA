#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Performance des listes sur 20 000 messages (mesure à 100 000 consignée dans docs/ETAT/TESTS.md).

mod support;
mod support_mirror;
mod support_reading;

use std::time::{Duration, Instant};

use mira_core::reading::{ListQuery, list_messages};
use mira_core::storage::Database;
use mira_vocabulary::{MailboxId, PageRequest, StandardView, ViewRef};
use support_reading::bulk::database_with_bulk_messages;
use support_reading::queries::{profile, standard};

const MESSAGES: u32 = 20_000;
const BUDGET: Duration = Duration::from_millis(50);

/// Mesure une page de liste et vérifie qu'elle est pleine
fn timed_page(db: &Database, query: &ListQuery) -> Duration {
    let started = Instant::now();
    let page = list_messages(db, &profile(), query).unwrap();
    let elapsed = started.elapsed();
    assert_eq!(page.items.len(), 50);
    elapsed
}

/// Une page de chaque vue (boîte, unifiée, non lus, suivis) reste sous le budget sur 20 000 messages
#[test]
fn first_page_of_each_view_is_fast_on_20000_messages() {
    let db = database_with_bulk_messages(MESSAGES);
    let mailbox = ViewRef::Mailbox {
        mailbox: MailboxId::from_raw("box-1"),
    };
    let mut queries = vec![ListQuery::of(mailbox)];
    for view in [
        StandardView::Unified,
        StandardView::Unread,
        StandardView::Flagged,
    ] {
        queries.push(standard(view));
    }
    for query in &queries {
        let elapsed = timed_page(&db, query);
        eprintln!("{:?} : {elapsed:?}", query.view);
        assert!(elapsed < BUDGET, "{:?} a pris {elapsed:?}", query.view);
    }
}

/// Une page profonde (curseur après 200 pages) et une page filtrée restent sous le budget
#[test]
fn deep_and_filtered_pages_are_fast_on_20000_messages() {
    let db = database_with_bulk_messages(MESSAGES);
    let mut query = standard(StandardView::Unified);
    for _ in 0..200 {
        let page = list_messages(&db, &profile(), &query).unwrap();
        query.page = PageRequest {
            cursor: page.next_cursor,
            limit: 50,
        };
    }
    let deep = timed_page(&db, &query);
    eprintln!("page profonde : {deep:?}");
    assert!(deep < BUDGET, "page profonde : {deep:?}");
    let mut filtered = standard(StandardView::Unified);
    filtered.filters.unread_only = true;
    filtered.filters.from_contains = Some("bob".to_string());
    let elapsed = timed_page(&db, &filtered);
    eprintln!("filtrée : {elapsed:?}");
    assert!(elapsed < BUDGET, "page filtrée : {elapsed:?}");
}
