# Audit de sécurité MIRA — reste à faire

État au **2026-11-26**. Ce document est le point d'entrée pour reprendre le chantier dans une nouvelle session : il ne dépend d'aucune conversation. Lire d'abord `CLAUDE.md` à la racine (règles du projet), puis ce fichier, puis `CONSTATS.md` pour le détail d'un point.

## 1. Où est quoi

| Document | Contenu |
|---|---|
| `CONSTATS.md` | Les 12 constats : problème, contexte, correctif, statut daté. Source de vérité des décisions. |
| `AUDIT-SECURITE-CHECKLIST.md` | Checklist OWASP (A01 à A10, LLM, poste de travail) avec statut et preuve par ligne ; « Lacunes restantes » en bas. |
| `MODELE-DE-MENACES.md` | Attaquants, frontières de confiance, parades en place et manques. |
| `VERIFICATIONS-REELLES.md` | Procédure à exécuter sur Linux, Android, Windows, avec les 9 mails de `mails-pieges/`. |
| `A-FAIRE-WINDOWS.md` | Jeton du démon et tube nommé sous Windows (reporté). |
| `../../SECURITY.md` | Politique publique de signalement (signalement privé GitHub activé). |

## 2. Ce qui est fait (ne pas refaire)

Vue web et HTML d'origine durcis (constat 1) et **rebranchés** (retirés par erreur par `33173df`) ; désabonnement sans machine privée (2) ; dépendances corrigées **dans les deux verrous**, moteur et pont (3, 12) ; SBOM par release (3) ; plafond de clients du démon et éviction de la plus ancienne connexion en attente (6) ; arrière-plans CSS distants bloqués (11) ; trousseau de secours Android (lecture illisible refusée) ; CI épinglée (10, partie épinglage) ; `SECURITY.md`, modèle de menaces, mails piégés. Fusionné dans `dev` : tout ce qui précède **sauf** la branche `feat/SEC-02-plafond-demon-ci-epinglee` (éviction, CI épinglée, ce document) tant qu'elle n'est pas fusionnée.

## 3. À faire, par ordre de priorité recommandé

### 3.1 Garde-fou contre la divergence des deux `Cargo.lock` — FAIT (2026-11-26, branche `feat/SEC-03-garde-fou-verrous`)
- **Fait** : `tools/check-lock-sync` échoue si une dépendance présente dans les deux verrous n'a pas le même ensemble de versions ; test de l'outil `tools/test-check-lock-sync` (écrit et vu rouge d'abord), branché dans `ci.sh`. Le contrôle des avis du pont (`cargo deny --manifest-path bridge/Cargo.toml --config engine/deny.toml check advisories`) est aussi dans `ci.sh`, ignoré avec un message si le réseau est indisponible ; vérifié vert le 2026-11-26.
- **Écarts existants** : les verrous divergeaient déjà sur 30 crates (`cc`, `serde_json`, `zstd`, `zerocopy`, `windows-*`…, aucun avis ouvert). Ils sont consignés dans `tools/lock-sync-ecarts.txt` : seul un écart nouveau ou une consigne périmée fait échouer. **Reste** : aligner les verrous (`cargo update -p <crate> --precise <version>` dans les deux, puis retirer la ligne) ; non fait ici car cela touche les dépendances livrées et demande un build et des tests complets.

### 3.2 Alerte d'hameçonnage sur les liens (constat 8, gros)
- **Pourquoi** : rien ne prévient quand le texte d'un lien affiche `banque.fr` et que la cible est `banque-secure.xyz`, ni pour les homographes (alphabets mélangés, IDN).
- **Faire** : comparer texte visible et domaine cible avant d'ouvrir un lien ; afficher la vraie adresse et demander confirmation en cas d'écart ; avertissement quand l'authentification de l'expéditeur échoue. Côté moteur (analyse des liens, `mira-core/src/html`) et côté Flutter (`openLink` dans `html_body.dart`, `web_mail_body.dart`).
- **Attention** : deux chemins d'affichage (widgets natifs et vue web) ; le lien cliqué dans la vue web passe par `_onNavigate` (`web_mail_body.dart`), dans les widgets par `onTapUrl: openLink`. Prévoir les liens de suivi légitimes (fausses alertes) et un test avec les mails piégés.
- **Terminé quand** : tests moteur (comparaison de domaines, homographes) et tests widget des deux chemins ; mail piégé dédié ajouté à `mails-pieges/`.

### 3.3 Consommation CPU de la vue web (signalée par l'utilisateur, à reprendre)
- **Constat** : la vue web (vrai moteur de navigateur) consomme nettement plus de CPU que les widgets ; remarqué à l'usage le 2026-11-26.
- **Pistes** : élargir la définition du mail « simple » (`isSimpleHtml` dans `app/lib/ui/body_renderer.dart`, seuils `_maxSimpleLength`, `_maxSimpleImages`, `_maxSimpleTables`, `_heavyTags`, `_heavyCss`) pour envoyer moins de mails à la vue web ; libérer la vue plus vite quand le mail n'est plus visible (voir `web_page_pause_test.dart` et les commits de pause des animations).
- **Terminé quand** : une mesure avant/après (CPU sur un mail riche, par exemple LinkedIn) est consignée.

### 3.4 Instruire les points 🟡 de la checklist — FAIT, un seul reste (2026-11-26, branche `feat/SEC-05-checklist-orange`)
28 points sur 29 sont passés à ✅ avec preuve citée dans `AUDIT-SECURITE-CHECKLIST.md`. Trouvailles pendant l'instruction : constat 13 (`SecretValue` non effacé, corrigé) et constat 14 (image distante autorisée vers une machine locale, corrigé en partie). **Reste 🟡 : SEC-A10-5** (le WebView préfetche-t-il le DNS des liens ?), non démontrable sans WebView : procédure de capture DNS ajoutée à `VERIFICATIONS-REELLES.md`, à exécuter par l'utilisateur. Choix à confirmer : aperçu des notifications activé par défaut (D-4) ; aucun verrou applicatif (A07-5) ; unité systemd non durcie, justification dans A05-3.

### 3.5 Mineurs — traités le 2026-11-26 (branche `feat/SEC-04-mineurs`)
- **Filtre d'adresses du désabonnement** : FAIT, Rust et Dart alignés, un test par plage (voir constat 2).
- **Rebinding DNS** : FAIT (`connectionFactory` sur l'adresse contrôlée, TLS sur le nom, test avec certificat local).
- **Poignée de main du démon** : FAIT le 2026-11-26 (SEC-06) : `hello_err busy`, protocole 1.2 (`DESIGN-BUSY.md`).
- **Vue web** : FAIT côté code (clics relayés par le script, navigation automatique bloquée) ; **à vérifier à la main** avec les mails piégés (ligne ajoutée à `VERIFICATIONS-REELLES.md`), le test unitaire ne peut pas piloter un vrai WebView.
- **Tests de l'éviction** : FAIT, synchronisation explicite sur l'inscription des connexions.

### 3.6 À exécuter par l'utilisateur
- **Vérifications réelles** : `VERIFICATIONS-REELLES.md` avec `tools/deposer-mails-pieges` (serveur IMAP local sur `127.0.0.1` seulement). Consigner OK ou ÉCHEC par ligne et par plateforme dans `RESULTATS-VERIFICATIONS.md` (à créer). Un ÉCHEC est une faille : correctif, test de non-régression, nouvelle vérification.
- **Audit Dart** : déclencher à la main le workflow `securite` sur GitHub (onglet Actions, « Run workflow ») ; traiter les avis sur `app/pubspec.lock`.
- **Premier passage de la CI GitHub après l'épinglage** : vérifier que les SHA épinglés fonctionnent (les workflows n'ont pas pu être lancés depuis la machine de développement).

## 4. Reporté, avec déclencheur de reprise

| Sujet | Raison du report | Reprendre quand |
|---|---|---|
| DKIM pour le « un clic » du désabonnement (constat 2, point 2) | gros chantier (vérification de signature, clés DNS, mails non signés) pour un risque déjà réduit | MIRA vérifie DKIM pour une autre raison (badge d'authentification, alerte du constat 8) |
| Fuzzing (MIME, IMAP, HTML, YAML, protocole du démon) | demande la toolchain nightly et de l'infrastructure ; aucun problème démontré | après la livraison des corrections actuelles, en chantier à part |
| Journal d'audit | n'empêche aucune attaque ; risque d'y écrire des données sensibles ; demande une conception | usage multi-utilisateur ou besoin de diagnostic |
| ACL Windows, tube nommé (constat 5) | demande une machine Windows à deux comptes ; un correctif non testé peut empêcher le démon de démarrer | une machine Windows à deux comptes, ou diffusion sur postes partagés (`A-FAIRE-WINDOWS.md`) |
| Signature des releases et clé Android propre (constat 10) | demande une clé à conserver hors du dépôt ; gain faible tant que MIRA reste installé par l'auteur | première diffusion publique, ou un tiers qui installe MIRA |
| Chiffrement de la base (SQLCipher) | risque accepté, documenté ; gain partiel pour un coût réel | exigence d'un public ou d'un audit externe (retour arrière possible) |

## 5. Décisions prises (journal)

| Date | Décision |
|---|---|
| 2026-11-26 | Constat 4 : base non chiffrée, documentée (`README.md`, `SECURITY.md`), disque à chiffrer. |
| 2026-11-26 | Constat 5 : Windows reporté. |
| 2026-11-26 | Reports : DKIM, fuzzing, journal d'audit. |
| 2026-11-26 | Vue web rebranchée (retrait accidentel de `33173df`) ; mode Auto : widgets pour les mails simples, vue web pour les riches. |
| 2026-11-26 | Branche `feat/SEC-01-audit-securite` non poussée ; fusionnée `--no-ff` dans `dev`, puis `dev` poussé. |
| 2026-11-26 | Plafond du démon : évincer la plus ancienne connexion en attente (`server/pending.rs`, `session/greeting.rs`). |
| 2026-11-26 | SEC-04 : navigation de la vue web — clics relayés par le script plutôt qu'un indicateur de geste ; refus `busy` en design seulement. |
| 2026-11-26 | Refus `busy` accepté et codé (SEC-06) ; aperçu des notifications gardé par défaut ; alignement des verrous à faire ensuite. |
| 2026-11-26 | SEC-05 : unité systemd `--user` non durcie (le démon lance l'app, `NoNewPrivileges` hériterait) ; `SecretValue` effacé à la destruction ; image distante vers machine privée refusée même autorisée. |
| 2026-11-26 | Constat 10 : épingler la CI maintenant ; signature des releases reportée. |

## 6. Pour reprendre

- **Règles** (`CLAUDE.md` du projet et du compte) : conversation et commits en français ; un chantier = un worktree + une branche `feat/<id>-<sujet>` créée depuis `dev`, jamais de travail direct sur `dev` ; test d'abord, vu en échec, puis le correctif à la cause racine ; **jamais supprimer ni affaiblir un test** pour passer au vert (un changement de règle décidé se documente et se commite avec la raison) ; tests ciblés pendant le travail ; `./ci.sh` complète **seulement avec l'accord de l'utilisateur** ; push, fusion dans `dev` et suppression sont soumis à confirmation ; pas de trailer `Co-Authored-By` dans les commits.
- **Normes de code Rust** (`tools/check-norms`) : fonction ≤ 30 lignes, fichier ≤ 6 fonctions (tests compris), une ligne `///` avant chaque fonction, aucun commentaire dans un corps de fonction, zéro `unwrap`/`expect`/`panic!` hors tests.
- **Tests ciblés utiles** : moteur `cargo test -p mira-core --test <fichier>` ; démon `cargo test -p mira-daemon --test session_pending_eviction` ; app `flutter test test/html_*_test.dart test/web_*_test.dart` ; outils `tools/test-check-ci-pins`, `tools/test-generate-sbom`. Limiter les tâches : `CARGO_BUILD_JOBS=4`.
- **Deux verrous Cargo** : toute mise à jour de dépendance se fait dans `engine/Cargo.lock` **et** `bridge/Cargo.lock`.
- **Construire et installer un essai local** : `CARGO_BUILD_JOBS=4 packaging/build-bundle.sh`, puis `packaging/tarball.sh`, `sha256sum` dans `packaging/out/SHA256SUMS`, puis `MIRA_INSTALL_FROM=$PWD/packaging/out packaging/install.sh --force --yes 2.4.0` ; redémarrer le démon (`systemctl --user restart mira-daemon.service`). Ne pas publier ce paquet : il embarque les identifiants OAuth locaux.

## 7. Nettoyage à faire

- Worktrees et branches locales une fois fusionnées : `feat/SEC-01-audit-securite` (worktree `.claude/worktrees/audit-securite`), puis `feat/SEC-02-plafond-demon-ci-epinglee` (worktree `.claude/worktrees/sec-02`). Ne pas toucher aux `git stash` existants : ils appartiennent à d'autres chantiers (`feat/E-02-html-render`).
