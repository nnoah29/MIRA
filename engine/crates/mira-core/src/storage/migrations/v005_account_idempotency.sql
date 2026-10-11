-- Clés d'idempotence de l'ajout de compte (12.2.3) : la même clé redonne le même compte.
-- Ce ne sont pas des secrets ; la ligne disparaît avec le compte.
CREATE TABLE account_idempotency (
    key TEXT PRIMARY KEY,
    address TEXT NOT NULL,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE
);
