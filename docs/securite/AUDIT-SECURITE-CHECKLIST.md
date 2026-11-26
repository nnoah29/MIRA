# Audit de sécurité MIRA — checklist OWASP

Branche `feat/SEC-01-audit-securite` (depuis `dev` @ 947f4af). Référentiels : OWASP Top 10:2021, OWASP Top 10 for LLM Applications, OWASP ASVS (sections applicables), OWASP MASVS (Android), chaîne d'approvisionnement.

## Légende

- ❌ **accepté** : lacune connue, risque assumé par décision datée.

- ✅ **Couvert** : mécanisme présent, preuve citée (reste à confirmer par un test dédié si non indiqué).
- 🟡 **À vérifier** : mécanisme probable ou partiel, lecture de code approfondie nécessaire.
- ❌ **Non couvert** : lacune constatée.

> **Portée de cette première passe** : sondage statique ciblé (recherches par motifs, lecture des fichiers clés). Ce n'est pas une revue exhaustive ; chaque 🟡 est une tâche d'audit à instruire. Aucun test dynamique n'a été lancé.

## Surface d'attaque

| Surface | Entrées non fiables | Composants |
|---|---|---|
| Contenu des mails | HTML, MIME, pièces jointes, en-têtes, liens | `mira-core/{mime,html}`, `app/lib/ui/{web_mail_body,html_body}.dart` |
| Serveurs distants | réponses IMAP/SMTP/OAuth | `mira-core/{imap,smtp,auth}` |
| IPC local | trames du démon (socket Unix / TCP 127.0.0.1) | `mira-daemon`, `mira-protocol`, `mira-host` |
| Connecteurs IA | réponses de modèles, URL de base éditable, outils | `mira-ai-*`, `mira-core/ai` |
| Fichiers importés | règles YAML, étiquettes, archives, modèles locaux | `mira-core/{portability,rules,ai/local}` |
| Secrets | trousseau, jetons OAuth, clés IA | `mira-core/{ports/secrets,auth}`, `mira-daemon/token` |
| Distribution | paquets, installeur, identifiants OAuth embarqués | `packaging/`, `mira-assembly` |

## A01 — Contrôle d'accès défaillant

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A01-1 | Socket Unix en 0600, dossier en 0700 | ✅ | `transport/unix.rs:40`, `unix_files.rs:11-13` |
| SEC-A01-2 | Jeton local en fichier privé, refus si 0o077 | ✅ | `token/file.rs` `refuse_shared` |
| SEC-A01-3 | TCP lié à 127.0.0.1 seulement | ✅ | `transport/tcp.rs:18` |
| SEC-A01-4 | Comparaison du jeton en temps constant | ✅ | `token/mod.rs:23` : HMAC + `hmac::verify` (corrigé après lecture) |
| SEC-A01-5 | Contrôle du pair sur socket Unix (`SO_PEERCRED`, même UID) | ✅ | `transport/unix.rs:54-58` : `peer_cred()` (SO_PEERCRED) comparé à l'uid propriétaire de la socket, pair refusé sinon ; tests `transport_unix.rs` (même uid accepté, prédicat) — l'acceptation d'un autre uid n'est testée qu'au niveau du prédicat (demande root) ; 2026-11-26 |
| SEC-A01-6 | Sous TCP loopback : tout processus local de n'importe quel utilisateur peut tenter de se connecter | ✅ | Jeton de 256 bits (`token/mod.rs`, `file.rs`), comparaison en temps constant, 5 s et 8 Kio avant `hello`, 8 attentes au plus avec éviction (`session_hello_limits.rs`, `session_pending_eviction.rs`) : l'énumération est hors de portée. TCP n'est actif que sous Windows ou sur `--tcp-port` (`config/mod.rs:68`). Reste documenté : port squatté qui récolte le jeton (`PROTOCOLE-DEMON.md`, « Limite connue du TCP loopback ») ; 2026-11-26 |
| SEC-A01-7 | Autorisations par opération (une session « lecture » ne peut pas écrire/envoyer) | ✅ | Accepté par conception : `Auth` n'a qu'un `LocalToken` ; `Access` = `Full` ou `UpdateOnly` (`session/access.rs`). Le jeton est un fichier 0600 du même utilisateur que la base et le trousseau : un processus qui le lit peut déjà lire tout le reste, une portée par opération ne protégerait rien (modèle de menaces, « autre utilisateur local »). À reprendre si un second client moins privilégié apparaît ; 2026-11-26 |
| SEC-A01-8 | Isolation entre comptes/profils (accès croisé par identifiant) | ✅ | Aucun handler de message, d'action ou de brouillon ne prend d'`account_id` du client (`api/handlers/*` : 3 occurrences, inscription et listes via `engine.account_ids()`) ; les identifiants sont ceux de la base d'un seul profil, un profil = un utilisateur. Isolation des profils : `profile/paths.rs` (nom validé, `profile_paths.rs` 4 tests, dossiers 0700) ; 2026-11-26 |
| SEC-A01-9 | Opérations d'hôte (`Shutdown`, `RetryUnlock`) hors contrat d'API | ✅ | `protocol/auth.rs` `HostOp` |

## A02 — Défaillances cryptographiques

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A02-1 | TLS rustls, pas de `danger_accept_invalid_*` | ✅ | grep négatif ; `imap/connection/tls.rs` |
| SEC-A02-2 | Refus du clair (IMAP/SMTP/OAuth) | ✅ | `imap/connection/policy.rs`, `auth/loopback.rs:17` |
| SEC-A02-3 | Version TLS minimale (TLS 1.2+ explicite) | ✅ | rustls seul (TLS 1.2 et 1.3), aucun openssl ni native-tls dans `Cargo.lock` |
| SEC-A02-4 | STARTTLS : protection contre le *downgrade* / injection avant TLS | ✅ | IMAP : `starttls.rs` refuse le repli (`require_ok`), `into_inner` jette les octets en clair déjà lus, capacités relues après TLS ; SMTP : `Tls::Required` |
| SEC-A02-5 | Stockage des secrets dans le trousseau système | ✅ | `ports/secrets/keyring_store.rs` |
| SEC-A02-6 | Repli si trousseau indisponible : pas de secret en clair sur disque | ✅ | repli Android : AES-256-GCM, nonce neuf, clé du Keystore, 0600, écriture atomique (`file_secrets/sealed_file.rs`). Une erreur de lecture autre que « absent » est désormais refusée (corrigé, `unreadable_file_is_an_error_not_an_empty_keyring`) |
| SEC-A02-7 | Base SQLite et corps de messages chiffrés au repos | ❌ accepté | risque accepté, décision B du 2026-11-26 : non chiffré, documenté (`README.md`, `SECURITY.md`) |
| SEC-A02-8 | Identifiants OAuth embarqués : clé AES stockée à côté du chiffré | ❌ | `oauth_embedded.rs` : simple obfuscation, `client_secret` extractable du binaire. Acceptable pour client public + PKCE, à documenter et à ne pas présenter comme un secret |
| SEC-A02-9 | PKCE S256 + `state` aléatoire + redirection loopback | ✅ | `auth/pkce.rs`, `flow/start_key.rs` |
| SEC-A02-10 | Aléa cryptographique pour jetons | ✅ | `random_hex` ; confirmer la source (`getrandom`/`ring`) |
| SEC-A02-11 | Effacement mémoire des secrets (`zeroize`) | ✅ | Constat 13 : `SecretValue` s'efface à la destruction (`ports/secrets/value.rs`, `ports_secret_value_zeroize.rs`) ; copies hors de ce type non couvertes, risque accepté (constat 13) ; 2026-11-26 |

## A03 — Injection

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A03-1 | SQL paramétré | ✅ | `format!` SQL ne porte que des constantes (`COLUMNS`, nom de table) ; `?1` partout. À confirmer pour `introspection.rs:39` (nom de table) |
| SEC-A03-2 | Injection de commandes IMAP (noms de boîtes, critères de recherche) | ✅ | `quote`/`quote_mailbox` refusent CR/LF/NUL partout ; drapeaux en énumération fixe (lecture, 2026-11-26) |
| SEC-A03-3 | Injection CRLF dans en-têtes SMTP/MIME (From, Subject, Reply-To, noms de fichiers) | ✅ | `smtp/compose/sanitize.rs` ; tests `smtp_compose_validation`, `smtp_compose_attachment_injection` |
| SEC-A03-4 | XSS dans le rendu HTML (ammonia, schémas, `data:`, SVG, CSS) | 🟡 | `html/sanitize.rs` : schémas `http,https,mailto,cid,data` ; `data:` image hors SVG seulement. Tester corpus XSS, `<style>`, `srcset`, `<base>`, `<meta refresh>`, `<form>` |
| SEC-A03-5 | WebView (Linux WebKitGTK / Android) : JavaScript désactivé, navigation et sous-ressources bloquées | 🟡 | `web_mail_body.dart` : **aucun réglage de sécurité trouvé** (JS, `NavigationDelegate`, accès fichiers). À instruire en priorité |
| SEC-A03-6 | Ouverture de liens : liste blanche de schémas | ✅ | `html_body.dart` `openLink` (http/https/mailto) ; `Process.run` sans shell |
| SEC-A03-7 | Hameçonnage par liens (texte ≠ cible, homographes IDN) | ❌ | aucune alerte constatée |
| SEC-A03-8 | Injection de prompt via contenu de mail (IA) | ✅ | Même preuve que SEC-LLM01 : `ai_injection_trap_mail.rs` avec le texte exact de `mails-pieges/09-injection-prompt.eml` ; `ai_tools_loop_masking.rs:85` (fermeture de cadre) ; 2026-11-26 |
| SEC-A03-9 | Injection YAML des règles importées | ✅ | alias, doublons, taille, éléments, imbrication : `portability_envelope_*` |

## A04 — Conception non sécurisée

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A04-1 | Modèle de menaces écrit (STRIDE) | ✅ | `MODELE-DE-MENACES.md` |
| SEC-A04-2 | Limites de taille/temps/profondeur (MIME, littéraux IMAP, HTML, pièces jointes) | ✅ | IMAP : ligne 1 Mio, littéral 64 Mio, réponse 64 Mio, annonce refusée avant allocation, délai d'inactivité réarmé (`imap/protocol/limits.rs:12-14`, `imap_reader_limits.rs` 5 tests) ; HTML 2 Mio et imbrication 512 (`html/limits.rs`, `html_sanitize_limits.rs` 5 tests) ; MIME 50 Mio, 10 000 parties, 5 000 niveaux (`mime/message.rs:11`, `mime_structure_limits.rs`) ; pièce jointe locale 25 Mio lue en une ouverture (`drafts/attachments/local_file.rs:14,50-58`, `drafts_attachments.rs` 5 tests) ; import YAML 2 Mio, 500 éléments, alias refusés (`portability_envelope_*`) ; 2026-11-26 |
| SEC-A04-3 | Chargement distant bloqué par défaut (pixels espions, confidentialité) | ✅ | `html/remote.rs` `is_blocked_until_allowed` |
| SEC-A04-4 | Confirmation avant envoi/suppression irréversibles initiés par l'IA | ✅ | aucun outil d'envoi ni de suppression ; propositions validées par clic |
| SEC-A04-5 | Idempotence et reprise sur crash (envoi en double) | ✅ | invariants du projet |

## A05 — Mauvaise configuration de sécurité

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A05-1 | Permissions des fichiers de données, export, clés IA | ✅ | `profile/paths.rs` 0700, `export_file.rs` 0600, `key_file.rs` 0600 |
| SEC-A05-2 | Permissions Windows (ACL) pour jeton et données | ❌ | reporté le 2026-11-26 (`A-FAIRE-WINDOWS.md`), mention dans `SECURITY.md` |
| SEC-A05-3 | Durcissement des paquets (deb/rpm/AppImage : permissions, scripts post-install) | 🟡 | auditer `packaging/` |
| SEC-A05-4 | Android : `allowBackup`, `exported`, `usesCleartextTraffic`, capture d'écran | ✅ | manifeste lu : `allowBackup="false"`, une activité exportée, pas de clair ; preuve WebView sur appareil (`VERIFICATIONS-REELLES.md`) |
| SEC-A05-5 | Processus IA local : environnement vidé, E/S neutralisées | ✅ | `ai/local/process.rs` `env_clear`, `kill_on_drop` |
| SEC-A05-6 | Fichiers `SECURITY.md` et politique de divulgation | ✅ | `SECURITY.md` à la racine |

## A06 — Composants vulnérables et obsolètes

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A06-1 | `cargo-deny` : bans, licences, sources | ✅ | `ci.sh:40-42` |
| SEC-A06-2 | Avis de sécurité (RustSec) | ✅ | `cargo deny check advisories` + `.github/workflows/securite.yml` |
| SEC-A06-3 | Audit des dépendances Dart/Flutter (`pubspec.lock`) | ❌ | aucun outil constaté |
| SEC-A06-4 | Dépendances sensibles : `zip`, `tar`, `flate2`, `ammonia`, `reqwest`, `rustls`, `keyring` | 🟡 | suivre les avis ; épingler les versions |
| SEC-A06-5 | Mise à jour automatique des dépendances (Dependabot/Renovate) | ✅ | `.github/dependabot.yml` |
| SEC-A06-6 | SBOM publié avec les releases | ✅ | `tools/generate-sbom`, joint aux releases (`release.yml`, `release.sh`) |

## A07 — Identification et authentification

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A07-1 | OAuth2 PKCE pour Gmail/Outlook | ✅ | `auth/` |
| SEC-A07-2 | Écouteur de rappel loopback : port éphémère, usage unique, `state` vérifié, délai | ✅ | `start.rs` : `127.0.0.1:0`, `state` comparé en temps constant, délai, un `state` erroné n'abat pas le flux (`connection.rs`) |
| SEC-A07-3 | Renouvellement/révocation des jetons, révocation à la suppression de compte | ✅ | Suppression de compte : caches, puis mot de passe, jeton d'accès, jeton de rafraîchissement et expiration (`accounts/deletion.rs`, `auth/vault/delete.rs`) ; reprise après interruption ; tests `auth_vault.rs:88` (les autres comptes intacts), `api_accounts_lifecycle.rs` (S14), `storage_no_secret.rs` (aucun secret en base). **Limite connue** : le serveur n'est jamais contacté (S14), le jeton OAuth n'est donc pas révoqué chez le fournisseur ; l'utilisateur peut le faire dans les réglages de son compte ; 2026-11-26 |
| SEC-A07-4 | Mots de passe d'application (Yahoo, générique) : stockage et journalisation | ✅ | Mot de passe rangé dans le trousseau et relu pour la connexion, `Debug` = `***` (`auth_vault.rs:17`), jamais en base (`storage_no_secret.rs`, invariant 15), jamais journalisé : les `tracing::` du moteur ne portent qu'un code d'erreur (grep sur `engine/crates/*/src`, aucun secret) ; `auth/form.rs` n'est qu'un encodeur de formulaires, la correspondance de la checklist était erronée ; 2026-11-26 |
| SEC-A07-5 | Verrouillage de l'application / déverrouillage trousseau | ✅ | Trousseau verrouillé : annoncé `keyring_locked`, nouvel essai à 5 s puis 10, 20… 60 s et aussitôt sur `retry_unlock`, arrêt propre pendant l'attente (`mira-daemon/src/unlock.rs`, `keyring_locked.rs` 4 tests, `keyring_locked_stop.rs` 2 tests). **Choix** : pas de verrou applicatif propre à MIRA ; la protection d'un poste laissé ouvert est le verrouillage de session du système et du trousseau (même ligne que le constat 4) ; 2026-11-26 |

## A08 — Intégrité des logiciels et des données

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A08-1 | Téléchargement de modèles/serveur IA : somme SHA-256 vérifiée | ✅ | `ai/local/download_verify.rs` ; confirmer que la somme n'est pas lue depuis la même source que le fichier |
| SEC-A08-2 | Extraction d'archives : *zip-slip*, liens symboliques, bombes de décompression | ✅ | `extract_check.rs`, `max_bytes` ; ajouter tests adverses |
| SEC-A08-3 | Signature des paquets et des releases | ❌ reporté | aucune signature des paquets ni des releases : `SHA256SUMS` vient de la même source ; APK signés avec la clé de debug. Décision du 2026-11-26 : reporté, déclencheur = première diffusion publique (constat 10) |
| SEC-A08-4 | Mise à jour du logiciel (canal, signature) | ✅ | aucune mise à jour automatique ; `install.sh` manuel, somme SHA-256 vérifiée |
| SEC-A08-5 | Import/export de profil : intégrité, chemins, écrasement | ✅ | L'import est celui des libellés et des règles en YAML : taille 2 Mio, 500 éléments, alias et clés doubles refusés, version et type vérifiés, références inconnues refusées (`portability/envelope.rs`, `scan.rs`, 23 tests `portability_*`), aucun chemin dans le fichier, aucune action d'envoi ou de transfert possible dans une règle (`RuleAction`). L'export du profil (`maintenance/profile_export.rs`) ne contient ni secret ni miroir ; fichier 0600 créé en exclusif puis renommé, un lien symbolique planté est remplacé, jamais suivi (`export_file.rs`, `maintenance_export_file.rs` 3 tests) ; 2026-11-26 |
| SEC-A08-6 | Épinglage des actions CI et permissions minimales du workflow | ✅ | actions épinglées par SHA, outils à version fixe, OSV-Scanner vérifié par somme ; `tools/check-ci-pins` dans `ci.sh` |

## A09 — Journalisation et supervision

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A09-1 | Aucun secret dans journaux/erreurs | ✅ | `Debug` masqué de `Auth` ; test d'absence de secrets (règle projet) |
| SEC-A09-2 | Journaux `tracing` : pas d'adresses, objets, corps de mails ni URL avec jeton | ✅ | journaux : codes d'erreur et textes fixes ; seuls les noms de boîtes apparaissent (`sync/download/account.rs`, `sync/strategy/dispatch.rs`), ni corps, ni adresses, ni jetons |
| SEC-A09-3 | Journal d'audit des événements sensibles (échecs d'authentification IPC, ajout de compte, envoi) | ❌ | non constaté |
| SEC-A09-4 | Rapports de plantage / télémétrie sans données personnelles | 🟡 | confirmer qu'il n'y en a pas |
| SEC-A09-5 | Sortie Flutter : `print`/`debugPrint` de données de mail | 🟡 | grep `print(` sans résultat ; vérifier en profondeur |

## A10 — SSRF et requêtes sortantes

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A10-1 | URL de base IA éditable (`base_url_editable`) : refus de `file:`, HTTP clair, adresses privées/métadonnées cloud | ✅ | `ai/base_url.rs` : refus de `file:`, clair hors boucle locale, identifiants ; LAN en HTTPS permis |
| SEC-A10-2 | Redirections HTTP (suivre, limiter, ne pas transmettre la clé inter-hôtes) | ✅ | `ai/transport/client.rs` : `Policy::none()` |
| SEC-A10-3 | Chargement d'images distantes : pas d'accès aux adresses locales | ✅ | Constat 14 : image distante autorisée vers une machine locale ou privée retirée par les deux assainisseurs (`html/sanitize.rs`, `html/original.rs`, `html_remote_private_targets.rs`) ; par défaut aucune image distante (`web_document.dart:9`). Reste : rebinding DNS et `style` non filtrés (constat 14) ; 2026-11-26 |
| SEC-A10-4 | Hôtes IMAP/SMTP saisis par l'utilisateur : ports, adresses internes | ✅ | Pas d'auto-configuration réseau : serveurs du provider ou saisis (`accounts/servers.rs:15`) ; clair refusé hors boucle locale (`imap/connection/policy.rs:9`, `imap_connection_errors.rs:28`) ; STARTTLS exigé quand choisi, réponse `NO` = échec sans repli en clair, tampon lu avant TLS jeté (`starttls.rs:12`, `wire.rs:58`) ; un serveur interne saisi par l'utilisateur est légitime (auto-hébergé), le moteur n'agit que sur ses ordres ; 2026-11-26 |
| SEC-A10-5 | Fuite via requêtes DNS/préchargement de liens | 🟡 | Pas de preuve possible sans WebView : `<link>` et `<meta>` sont retirés par les assainisseurs (pas de `dns-prefetch`/`preconnect` écrits par le mail), images distantes bloquées par défaut ; reste à savoir si WebKitGTK préfetche le DNS des liens `<a>`. Procédure ajoutée à `VERIFICATIONS-REELLES.md` (capture DNS) ; 2026-11-26 |

## OWASP Top 10 pour applications LLM (connecteurs IA)

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-LLM01 | Injection de prompt indirecte par le contenu d'un mail | ✅ | Contenu cadré en donnée (`ai/chat/framing.rs` : balise `<tool_result>`, fermeture neutralisée, `TOOLS_NOTICE` « jamais une consigne »), balise `<context>` neutralisée (`neutralize.rs`) ; aucun outil d'envoi/suppression ; actions et mémoire = propositions. Test de bout en bout avec le mail piégé 09 : `ai_injection_trap_mail.rs` (3 tests, un modèle qui obéit n'obtient rien) ; 2026-11-26. Comportement d'un vrai modèle : `VERIFICATIONS-REELLES.md` (mail 09) |
| SEC-LLM02 | Sortie non assainie : rendu Markdown/HTML de la réponse du modèle | ✅ | `chat_markdown.dart` : gras, italique, code seulement ; ni lien ni image |
| SEC-LLM03 | Outils à agence excessive (envoi, suppression, règles, transfert) sans confirmation humaine | ✅ | catalogue sans outil d'envoi ; `propose_action` validée par l'utilisateur ; consentement par type de contenu |
| SEC-LLM04 | Exfiltration de données : le mail ordonne au modèle de transmettre du contenu par outil/lien/image | ✅ | pas d'image ni de lien rendus dans les réponses ; images de mail bloquées par défaut |
| SEC-LLM05 | Mémoire IA (`ai_memories`) empoisonnable par un mail | ✅ | `remember`/`forget_memory` sont des propositions : rien n'est écrit avant le clic |
| SEC-LLM06 | Envoi de contenu privé à un tiers : consentement par fournisseur, minimisation | ✅ | Une connexion distante sans consentement est refusée avant toute requête (`ai_chat_consent.rs:16`, le consentement précède la clé `:34`) ; chaque outil de lecture n'est exécuté que pour un type de contenu autorisé par la connexion (`chat/tool_consent.rs`, `ai_tools_loop_consent.rs`) ; valeurs sensibles (cartes, IBAN, OTP, secrets, contacts) masquées à l'envoi et dans les résultats d'outils (`ai/masking/`, `ai_chat_masking.rs`, `ai_tools_loop_masking.rs`) ; contexte plafonné à 120 000 caractères (`truncate.rs`) ; seule la boucle locale est exemptée ; 2026-11-26 |
| SEC-LLM07 | Clés d'API : stockage, journalisation, envoi seulement à l'hôte prévu | ✅ | Clé : trousseau système (`secrets`), jamais en journal, erreur ni `Debug` (`ai_chat_secrets.rs`, 3 tests dont rejet de clé sans fuite) ; en-tête marqué sensible (`transport/auth_header.rs:30`) ; envoyée à l'URL de la connexion seule : HTTPS hors boucle locale, sans identifiants dans l'URL (`base_url.rs`), aucune redirection (`transport/client.rs:30`, `ai_transport.rs:99` : la cible ne reçoit pas la clé) ; clé du serveur local tirée à chaque lancement, fichier 0600 (`local/key_file.rs`) ; 2026-11-26 |
| SEC-LLM08 | Déni de portefeuille / boucles d'outils : plafonds de jetons et d'itérations | ✅ | Au plus 8 tours (`chat/rounds.rs:13`) puis `MaxToolRounds` (`ai_tools_loop_limits.rs`), 8 appels par tour (`tool_step.rs:10`), outils de lecture retirés après 4 tours, entrée plafonnée à 120 000 caractères, sortie 4 096 jetons par défaut (`wire/*_encode.rs`), résultat d'outil borné (`ai_tools_loop_hardening.rs:32`) ; la valeur `max_output_tokens` vient du client, c'est-à-dire de l'utilisateur ; 2026-11-26 |
| SEC-LLM09 | Modèles locaux : provenance et intégrité | ✅ | voir SEC-A08-1 |

## Spécifique poste de travail et mobile

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-D-1 | Pièces jointes : nom assaini, pas d'exécution automatique, avertissement sur types exécutables | ✅ | Nom : seul le dernier composant, sans contrôle ni inversion bidi, 255 octets (`mime/attachment_name.rs:26`, `mime_attachment_names.rs`), puis `safeFileName` et `freeName` côté app (jamais d'écrasement : `app/lib/data/attachment_save.dart`) ; écriture atomique par temporaire exclusif (`fetching/atomic_write.rs`) ; aucune ouverture ni exécution de pièce jointe dans l'app (aucun `Process`/ouverture de fichier hors liens http, https, mailto de `openLink`) ; fichier enregistré sans bit exécutable (umask). Pas d'avertissement sur les types exécutables : sans objet tant que l'app n'ouvre rien ; 2026-11-26 |
| SEC-D-2 | Écriture atomique et suivi des liens symboliques dans les dossiers de destination | ✅ | `atomic_write.rs` ; test de lien symbolique à ajouter |
| SEC-D-3 | Presse-papiers : données sensibles | 🟡 | `original_dialog.dart` copie du texte |
| SEC-D-4 | Notifications : contenu affiché écran verrouillé | 🟡 | `mira-host/notifier` |
| SEC-D-5 | Dictée vocale : audio traité localement, pas d'envoi non annoncé | 🟡 | `docs/VOICE.md` |
| SEC-D-6 | Fuzzing des analyseurs (MIME, IMAP, HTML, YAML, protocole démon) | ❌ | aucun dossier `fuzz` |
| SEC-D-7 | Désérialisation des trames du démon : taille maximale, version, fuzz | ✅ | Taille jugée sur l'en-tête avant lecture du corps (`codec.rs:body_len`, `wire.rs:46-60` lit au plus `length` octets, mémoire croissante au fil des octets) ; `hello` ≤ 8 Kio ; tests `codec.rs` (6), `wire.rs`, `session_limits.rs`, `session_hello_limits.rs` verts ; profondeur JSON bornée par `serde_json` (128). Le fuzzing reste reporté (SEC-D-6) ; 2026-11-26 |

## Lacunes restantes (❌)

1. **SEC-A08-3** — releases non signées, reporté (constat 10).
2. **SEC-A06-3** — audit des dépendances Dart pas encore exécuté (workflow en place, OSV-Scanner absent localement).
3. **SEC-A03-7** — pas d'alerte d'hameçonnage sur les liens.
4. **SEC-A09-3** — pas de journal d'audit, reporté (2026-11-26).
5. **SEC-A05-2** — ACL Windows absentes, reporté (`A-FAIRE-WINDOWS.md`).
6. **SEC-D-6** — pas de fuzzing, reporté (2026-11-26).
7. **DKIM pour le désabonnement** (constat 2, point 2), reporté (2026-11-26).

Corrigées ou documentées : SEC-A06-2, SEC-A06-6 (SBOM), SEC-A02-7 (risque accepté, documenté), SEC-A02-8 (documenté), SEC-A04-1, SEC-A05-6.

## Ordre d'instruction proposé

1. **Priorité haute** : SEC-A03-5 (WebView), SEC-A03-4 (assainisseur HTML), SEC-LLM03/04 (agence et exfiltration IA), SEC-A03-3 (CRLF), SEC-A01-4/7 (jeton, portées), SEC-A06-2 (avis RustSec).
2. **Priorité moyenne** : SEC-A10-*, SEC-A02-4/6, SEC-A04-2, SEC-A07-2, SEC-A08-3/5, Android.
3. **Priorité basse / documentation** : modèle de menaces, `SECURITY.md`, SBOM, journal d'audit, fuzzing.

Chaque point instruit reçoit : constat, preuve (fichier:ligne), gravité (CVSS simplifié), correctif, test de non-régression.
