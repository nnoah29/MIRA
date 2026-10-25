-- Modèles de messages : textes réutilisables communs à l'app, avec variables {{…}} résolues par le moteur.
CREATE TABLE message_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    name_key TEXT NOT NULL UNIQUE,
    subject TEXT,
    body TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Clés d'idempotence des créations de modèles : la même clé rejouée rend le même résultat.
CREATE TABLE template_idempotency (
    key TEXT PRIMARY KEY,
    operation TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    result TEXT NOT NULL
);
