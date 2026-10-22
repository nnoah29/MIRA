-- Clés d'idempotence des écritures sur les libellés locaux (invariant 18) : la même clé rejouée rend le même résultat.
-- `result` est la réponse JSON de la première exécution ; pas de clé étrangère, une suppression rejouée retrouve sa clé.
CREATE TABLE local_label_idempotency (
    key TEXT PRIMARY KEY,
    operation TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    result TEXT NOT NULL
);
