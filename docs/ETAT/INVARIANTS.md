# Invariants et scénarios → tests

| # | Invariant | Test(s) |
|---|---|---|
| 1 | Le cœur ne nomme aucun provider (D-06) | garde-fou `tools/check-architecture` A3 + A6 (auto-test `tools/test-check-architecture`) |
| 2 | Un provider ne dépend que du vocabulaire (D-33) | garde-fou `tools/check-architecture` A2 (auto-test) |
| 3 | Provider sans I/O ni état (D-07) | garde-fou `tools/check-architecture` A2 (aucune dépendance d'I/O possible) + providers en structures unitaires |
| 4 | Aucune notion d'écran (P1) | garde-fou `tools/check-architecture` A9 |
| 5 | Bibliothèque externe via son module d'encapsulation (D-34) | garde-fou `tools/check-architecture` A8 (auto-test) |
| 6 | Tout ce que fait l'interface, le CLI le fait (P1) | `invariant_6_every_request_has_a_cli_command` (mira-cli) |
| 7 | Identifiant interne unique et stable (D-13) | `invariant_7_internal_id_stable_across_move_and_renumber` (mira-core)<br>`invariant_7_internal_id_survives_uid_change` (mira-core) |
| 8 | Message : au moins une apparition | `invariant_8_message_without_appearance_is_purged` (mira-core) |
| 9 | Apparition → boîte du même compte | `invariant_9_appearance_mailbox_same_account` (mira-core) |
| 10 | Message → un compte ; compte → un profil | `invariant_10_message_belongs_to_one_account_account_to_one_profile` (mira-core) |
| 11 | Compte : au moins une identité | `invariant_11_added_account_has_default_identity` (mira-core)<br>`invariant_11_set_default_identity_leaves_only_one` (mira-core) |
| 12 | Drapeaux utilisateur au niveau du message (D-14) | `invariant_12_flags_live_on_message` (mira-core)<br>`invariant_12_mark_read_applies_to_message_once` (mira-core) |
| 13 | Fil dérivé, recalculable (D-15) | `invariant_13_threads_recomputed_identically` (mira-core) |
| 14 | Source brute conservée, analyse en cache (D-16) | `invariant_14_raw_source_roundtrip_compressed` (mira-core) |
| 15 | Aucun secret en base, journaux, événements, API (P9) | `invariant_15_no_oauth_token_in_responses_events_or_database` (mira-core)<br>`invariant_15_no_secret_in_database` (mira-core)<br>`invariant_15_no_secret_in_events_errors_or_responses` (mira-core)<br>`invariant_15_no_secret_in_responses_events_or_database` (mira-core) |
| 16 | Miroir jetable sans perte (8.1) | `invariant_16_discard_mirror_keeps_config_and_intentions` (mira-core) |
| 17 | Action écrite avant d'être tentée | `invariant_17_action_persisted_before_attempt` (mira-core)<br>`invariant_17_action_written_before_attempt` (mira-core) |
| 18 | Action rejouée : pas deux effets | `invariant_18_enqueue_is_idempotent` (mira-core)<br>`invariant_18_replay_after_crash_has_single_effect` (mira-core)<br>`invariant_18_resubmission_with_same_key_has_single_effect` (mira-core) |
| 19 | Actions d'un compte rejouées dans l'ordre | `invariant_19_actions_replayed_in_order` (mira-core)<br>`invariant_19_actions_replayed_in_submission_order` (mira-core) |
| 20 | Repère avancé après écriture des données | `invariant_20_marker_advanced_after_data` (mira-core) |
| 21 | Le serveur gagne ; actions rejouées (D-23) | `invariant_21_server_wins_on_flags` (mira-core) |
| 22 | Erreur définitive : effet local annulé, utilisateur informé | `invariant_22_definitive_failure_reverts_local_effect` (mira-core)<br>`invariant_22_server_refusal_reverts_local_effect_and_informs` (mira-core) |
| 23 | Doute d'identité : garder deux messages (P7) | `invariant_23_ambiguous_fingerprint_keeps_two_messages` (mira-core) |
| 24 | Une tâche de synchro par compte | `invariant_24_burst_of_start_sync_requests_is_merged` (mira-core)<br>`invariant_24_single_sync_task_per_account` (mira-core) |
| 25 | Connexions ≤ plafond du provider | `invariant_25_connections_within_provider_limit` (mira-core) |
| 26 | Jamais deux envois sans décision de l'utilisateur | `invariant_26_interrupted_send_becomes_uncertain_never_resent` (mira-core)<br>`invariant_26_restart_makes_interrupted_send_uncertain_never_resent` (mira-core) |
| 27 | Aucune lecture ordinaire par le réseau (P5) | `invariant_27_list_and_read_work_offline` (mira-core)<br>`invariant_27_reads_never_touch_network` (mira-core) |
| 28 | Toute écriture rend la main sans réseau | `invariant_28_actions_return_while_offline` (mira-core)<br>`invariant_28_submit_returns_without_network` (mira-core)<br>`invariant_28_write_returns_without_network` (mira-core) |
| 29 | Toute liste paginée par curseur | `invariant_29_list_is_cursor_paginated` (mira-core)<br>`invariant_29_lists_are_paginated_through_the_facade` (mira-core)<br>`invariant_29_unified_view_paginates_without_duplicates` (mira-core) |
| 30 | Identifiants exposés opaques | `invariant_30_ids_are_opaque_strings` (mira-core) |
| 31 | Erreur : catégorie, code stable, réessayable | `invariant_31_error_codes_are_unique_and_prefixed_by_category` (mira-vocabulary) |
| 32 | Événements numérotés, sans secret ni contenu | `invariant_32_events_are_numbered_and_persisted` (mira-core)<br>`invariant_32_subscription_resumes_after_seen_number_then_follows_live` (mira-core) |
| 33 | Version du contrat vérifiée à l'ouverture | `invariant_33_contract_version_compatibility` (mira-vocabulary)<br>`invariant_33_incompatible_client_version_is_refused` (mira-core) |

| Scénario | Description | Test(s) |
|---|---|---|
| S1 | Ajouter un compte Gmail par OAuth | `scenario_s1_add_oauth_account` (mira-core) |
| S2 | Ajouter un compte Yahoo (mot de passe d'application) | `scenario_s2_add_app_password_account` (mira-core) |
| S3 | Ajouter un compte « Autre » | `scenario_s3_add_other_account_with_manual_servers` (mira-core) |
| S4 | Première synchronisation | `scenario_s4_add_account_runs_first_sync_inbox_first_with_progress` (mira-core)<br>`scenario_s4_first_sync_downloads_inbox_first_with_progress` (mira-core) |
| S5 | Nouveau message en temps réel | `scenario_s5_engine_idle_new_message_emits_event` (mira-core)<br>`scenario_s5_idle_new_message_emits_event` (mira-core) |
| S6 | Lire un message hors ligne | `scenario_s6_read_body_offline_through_the_facade` (mira-core)<br>`scenario_s6_read_offline_body_present_or_unavailable` (mira-core) |
| S7 | Déplacer des messages hors ligne | `scenario_s7_move_offline_then_replay_in_order` (mira-core)<br>`scenario_s7_move_without_move_extension_copies_then_expunges` (mira-core) |
| S8 | Message supprimé ailleurs | `scenario_s8_action_on_deleted_message_fails_and_reverts` (mira-core) |
| S9 | Envoyer, avec annulation | `scenario_s9_send_with_undo_then_send` (mira-core) |
| S10 | Jeton expiré en pleine opération | `scenario_s10_token_expired_keeps_actions_queued` (mira-core) |
| S11 | Message Gmail à plusieurs libellés | `scenario_s11_message_in_two_labels_is_one_message` (mira-core)<br>`scenario_s11_unified_view_lists_each_message_once` (mira-core) |
| S12 | Remonter au-delà de 90 jours, chercher | `scenario_s12_extend_window_downloads_older_headers` (mira-core)<br>`scenario_s12_extend_window_through_facade_downloads_older_headers` (mira-core) |
| S13 | Le serveur renumérote une boîte | `scenario_s13_resync_mailbox_through_facade_keeps_internal_ids` (mira-core)<br>`scenario_s13_uidvalidity_change_resynchronizes_mailbox` (mira-core) |
| S14 | Supprimer un compte | `scenario_s14_delete_account_keeps_others_and_server` (mira-core)<br>`scenario_s14_delete_interrupts_sync_and_keeps_other_account_running` (mira-core)<br>`scenario_s14_interrupted_deletion_is_completed_at_startup` (mira-core) |
