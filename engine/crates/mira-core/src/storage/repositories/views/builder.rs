//! Construction d'une requête de liste de messages : clauses et valeurs liées par numéro.

use rusqlite::types::Value;

/// Requête en cours de construction ; chaque valeur liée reçoit un paramètre numéroté.
#[derive(Debug)]
pub(super) struct QueryBuilder {
    from: String,
    clauses: Vec<String>,
    values: Vec<Value>,
}

impl QueryBuilder {
    /// Démarre une requête sur la source `from`, qui doit nommer la table des messages `m`
    pub(super) fn new(from: &str) -> Self {
        Self {
            from: from.to_string(),
            clauses: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Lie une valeur et rend son paramètre numéroté (`?3`)
    pub(super) fn bind(&mut self, value: impl Into<Value>) -> String {
        self.values.push(value.into());
        format!("?{}", self.values.len())
    }

    /// Lie une liste de textes et rend leur liste de paramètres (`(?1, ?2)`)
    pub(super) fn bind_list(&mut self, items: &[&str]) -> String {
        let names: Vec<String> = items
            .iter()
            .map(|item| self.bind(item.to_string()))
            .collect();
        format!("({})", names.join(", "))
    }

    /// Ajoute une condition, combinée aux autres par « et »
    pub(super) fn require(&mut self, clause: impl Into<String>) {
        self.clauses.push(clause.into());
    }

    /// Termine la requête : tri par date décroissante puis identifiant, limite liée
    pub(super) fn finish(mut self, columns: &str, limit: i64) -> (String, Vec<Value>) {
        let limit = self.bind(limit);
        let conditions = match self.clauses.is_empty() {
            true => String::new(),
            false => format!(" WHERE {}", self.clauses.join(" AND ")),
        };
        let sql = format!(
            "SELECT {columns} FROM {}{conditions} ORDER BY m.sort_date DESC, m.id DESC LIMIT {limit}",
            self.from
        );
        (sql, self.values)
    }
}
