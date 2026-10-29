-- Journal des envois à un service d'IA : une ligne par génération, des comptes et des codes, jamais de contenu.
CREATE TABLE ai_journal (
    id TEXT PRIMARY KEY,
    at INTEGER NOT NULL,
    operation TEXT NOT NULL UNIQUE,
    connection TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    local INTEGER NOT NULL,
    kinds TEXT NOT NULL,
    parts INTEGER NOT NULL,
    chars INTEGER NOT NULL,
    masked INTEGER NOT NULL DEFAULT 0,
    input_tokens INTEGER,
    output_tokens INTEGER,
    outcome TEXT NOT NULL
);

CREATE INDEX ai_journal_by_date ON ai_journal(at);
