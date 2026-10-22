-- Libellés locaux (R2) : libellés communs à tous les comptes, posés par conversation.
-- La pose suit (compte, clé du fil) et non l'identifiant du fil : les fils sont effacés puis recréés au recalcul.
CREATE TABLE local_labels (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    name_key TEXT NOT NULL UNIQUE,
    color INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (CAST(strftime('%s', 'now') AS INTEGER) * 1000)
);

CREATE TABLE thread_local_labels (
    label_id TEXT NOT NULL REFERENCES local_labels(id) ON DELETE CASCADE,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    root_key TEXT NOT NULL,
    PRIMARY KEY (label_id, account_id, root_key)
);
CREATE INDEX thread_local_labels_by_thread ON thread_local_labels(account_id, root_key);
