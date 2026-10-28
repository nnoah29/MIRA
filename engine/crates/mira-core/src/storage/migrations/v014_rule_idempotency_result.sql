-- Rapport d'un import de règles, rejoué tel quel quand la même clé revient (invariant 18).
ALTER TABLE rule_idempotency ADD COLUMN result TEXT;
