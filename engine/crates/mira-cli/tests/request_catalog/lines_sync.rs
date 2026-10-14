//! Lignes de commande du domaine synchronisation et de l'ajout OAuth combiné (invariant 6).

/// Au moins une ligne par variante de `SyncRequest`, plus les formes de portée et l'ajout OAuth.
pub const SYNC_LINES: &[&[&str]] = &[
    &["sync", "start"],
    &["sync", "start", "--account", "acc", "--wait"],
    &["sync", "start", "--mailbox", "box", "--timeout", "30"],
    &["sync", "pause", "--account", "acc"],
    &["sync", "status"],
    &["sync", "extend-window", "--mailbox", "box", "--days", "30"],
    &["sync", "resync", "--mailbox", "box"],
    &["account", "add-oauth", "--address", "a@b.c"],
    &[
        "account",
        "add-oauth",
        "--address",
        "a@b.c",
        "--provider",
        "gmail",
        "--sync-wait",
    ],
];
