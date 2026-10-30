-- Connexions aux services d'IA : adresse, modèle par défaut, consentement et liste de modèles gardée ; jamais de clé.
CREATE TABLE ai_connections (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL,
    label TEXT NOT NULL,
    base_url TEXT,
    default_model TEXT,
    is_default INTEGER NOT NULL DEFAULT 0,
    consent_at INTEGER,
    models_json TEXT,
    models_fetched_at INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Au plus une connexion par défaut.
CREATE UNIQUE INDEX ai_connections_single_default ON ai_connections(is_default) WHERE is_default = 1;
