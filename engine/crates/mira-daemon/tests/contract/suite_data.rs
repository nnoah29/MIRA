//! Macro qui instancie les scénarios de libellés locaux et de règles pour un chemin donné.

/// Génère les tests de ce thème pour ce `Driver`.
macro_rules! data_suite {
    ($driver:ty) => {
        /// Le cycle de vie d'un libellé local passe par le chemin.
        #[tokio::test(flavor = "multi_thread")]
        async fn local_label_lifecycle() {
            $crate::contract::labels::local_label_lifecycle::<$driver>().await
        }

        /// Créer, lister puis supprimer une règle.
        #[tokio::test(flavor = "multi_thread")]
        async fn rule_create_list_delete() {
            $crate::contract::rules::rule_create_list_delete::<$driver>().await
        }
    };
}

pub(crate) use data_suite;
