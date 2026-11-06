//! Macro qui instancie les scénarios de masquage, d'outils et de propositions pour un chemin donné.

/// Génère les tests de ce thème pour ce `Driver`.
macro_rules! chat_extras_suite {
    ($driver:ty) => {
        /// La ligne masked précède le texte.
        #[tokio::test(flavor = "multi_thread")]
        async fn masked_line_is_emitted_before_text() {
            $crate::contract::chat_masked::masked_line_is_emitted_before_text::<$driver>().await
        }

        /// Les lignes d'outil arrivent dans l'ordre.
        #[tokio::test(flavor = "multi_thread")]
        async fn tool_lines_arrive_in_order() {
            $crate::contract::chat_tools::tool_lines_arrive_in_order::<$driver>().await
        }

        /// Sans outils permis, aucune ligne d'outil.
        #[tokio::test(flavor = "multi_thread")]
        async fn start_without_allow_tools_has_no_tool_lines() {
            $crate::contract::chat_tools::start_without_allow_tools_has_no_tool_lines::<$driver>()
                .await
        }

        /// La proposition de brouillon arrive entre l'annonce et la fin de l'appel.
        #[tokio::test(flavor = "multi_thread")]
        async fn draft_proposed_line_arrives() {
            $crate::contract::chat_draft::draft_proposed_line_arrives::<$driver>().await
        }

        /// Le mode composeur vient du JSON de l'application.
        #[tokio::test(flavor = "multi_thread")]
        async fn composer_mode_comes_from_the_json() {
            $crate::contract::chat_draft::composer_mode_comes_from_the_json::<$driver>().await
        }

        /// Une action proposée arrive comme ligne action_proposed.
        #[tokio::test(flavor = "multi_thread")]
        async fn action_proposed_line_arrives() {
            $crate::contract::chat_draft::action_proposed_line_arrives::<$driver>().await
        }
    };
}

pub(crate) use chat_extras_suite;
