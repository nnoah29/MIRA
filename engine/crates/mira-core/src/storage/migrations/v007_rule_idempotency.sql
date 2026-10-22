-- Clés d'idempotence des écritures sur les règles (invariant 18) : la même clé rejouée rend le même effet.
-- Pas de clé étrangère : une suppression rejouée doit retrouver sa clé après la disparition de la règle.
CREATE TABLE rule_idempotency (
    key TEXT PRIMARY KEY,
    operation TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    rule_id TEXT
);
