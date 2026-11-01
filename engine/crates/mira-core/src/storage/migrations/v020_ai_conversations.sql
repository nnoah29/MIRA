-- Conversations avec l'assistant IA : un document par conversation, le payload est un texte JSON opaque pour le moteur.
CREATE TABLE ai_conversations (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    payload_version INTEGER NOT NULL,
    payload TEXT NOT NULL
);

CREATE INDEX ai_conversations_by_date ON ai_conversations(updated_at);
