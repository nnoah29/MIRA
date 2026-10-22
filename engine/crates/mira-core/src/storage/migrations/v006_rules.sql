-- Règles « condition → action » (R1) : configuration ordonnée, portée par compte, file des messages à évaluer.
CREATE TABLE rules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    enabled INTEGER NOT NULL,
    position INTEGER NOT NULL,
    scope_all INTEGER NOT NULL,
    schema_version INTEGER NOT NULL,
    definition TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (CAST(strftime('%s', 'now') AS INTEGER) * 1000),
    updated_at INTEGER NOT NULL DEFAULT (CAST(strftime('%s', 'now') AS INTEGER) * 1000)
);
CREATE INDEX rules_by_position ON rules(position);

CREATE TABLE rule_accounts (
    rule_id TEXT NOT NULL REFERENCES rules(id) ON DELETE CASCADE,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    PRIMARY KEY (rule_id, account_id)
);

CREATE TABLE rule_pending (
    message_id TEXT PRIMARY KEY
);
