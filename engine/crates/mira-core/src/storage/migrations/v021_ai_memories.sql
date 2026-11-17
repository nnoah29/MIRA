-- Mémoire de l'assistant IA : un souvenir par ligne, avec son vecteur (f32 little-endian) et le modèle qui l'a calculé.
CREATE TABLE ai_memories (
    id TEXT PRIMARY KEY,
    text TEXT NOT NULL,
    kind TEXT NOT NULL,
    source TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    model_id TEXT,
    embedding BLOB,
    dim INTEGER,
    stale INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX ai_memories_by_date ON ai_memories(created_at);
