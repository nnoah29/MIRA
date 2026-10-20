#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `mira search local` sans mot : les filtres seuls listent les messages, sans filtre : entrée invalide (code 10).

#[allow(dead_code)]
mod cli_support;

use cli_support::full_bench::FullBench;
use cli_support::json_views::page_ids;

/// Un filtre sans mot liste les messages concernés, hors ligne
#[tokio::test]
async fn a_filter_without_words_lists_messages() {
    let (bench, _account) = FullBench::start_seeded().await;
    bench.imap.shutdown();
    let found = bench.run_ok(&["search", "local", "--from", "carla"]).await;
    assert!(!page_ids(&found).is_empty(), "{found}");
    let nothing = bench.run(&["search", "local"]).await;
    assert_eq!(nothing.code, 10, "{}", nothing.stderr);
}
