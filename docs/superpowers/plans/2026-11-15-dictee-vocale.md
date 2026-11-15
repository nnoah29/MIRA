# Dictée vocale — plan d'implémentation

> **Pour les agents d'exécution :** utiliser `superpowers:subagent-driven-development` (recommandé) ou
> `superpowers:executing-plans`. Les étapes utilisent des cases `- [ ]`.

**Objectif :** dicter en direct, localement, en français et en anglais, dans le corps d'un brouillon et dans le champ du chat IA.

**Architecture :** un module Dart `app/lib/voice/` autonome : capture micro (`AudioSource`) → reconnaissance en flux dans un isolate (`SpeechRecognizer`, `sherpa_onnx` + modèles Kroko) → insertion dans un `TextEditingController` (`DictationInserter`), pilotée par un `DictationController` unique. Les modèles se téléchargent à la demande (`VoiceModelStore`). Le moteur Rust et le pont ne changent pas.

**Pile :** Flutter 3.47 / Dart 3.13, `provider`, `shared_preferences`, nouvelles dépendances `sherpa_onnx` 2.0.1, `record` 7.2.0, `archive` 4.7.1, `crypto` 3.0.8.

**Conception :** `docs/superpowers/specs/2026-11-15-dictee-vocale-design.md` (à lire avec ce plan).

## Contraintes globales

- Worktree `/home/nnoah/_WORKS/DeV/MIRA-voice`, branche `feat/voice-dictee-conversation` (issue de `dev`). Ne jamais changer de branche dans `/home/nnoah/_WORKS/DeV/MIRA` : un autre agent y travaille.
- Code dans `app/lib/voice/` (logique), `app/lib/ui/` (écrans), tests dans `app/test/voice_*_test.dart`.
- Chaque fichier Dart : `//!` de 1 à 3 lignes en tête ; une ligne `///` en français avant chaque fonction publique ou de classe ; identifiants en anglais.
- Tout texte visible passe par `t()` ; vérifier avec `python3 tools/i18n-audit` (doit rendre 0).
- Aucune donnée vocale ni texte dicté sur le réseau ; le seul accès réseau est le téléchargement du modèle, après confirmation de l'utilisateur.
- Les modèles ne sont jamais commités ni embarqués dans le binaire (licence CC-BY-SA).
- Tests uniquement contre `127.0.0.1` ; aucun test ne dépend du micro.
- Tests **ciblés** (`cd app && flutter test test/<fichier>`). Ne pas lancer `./ci.sh` ni la suite complète sans l'accord explicite de l'utilisateur ; avant le dernier commit, dire que la CI n'a pas tourné et la proposer.
- Commits : `feat(voix): résumé` (ou `test`, `docs`), un par tâche, au nom de l'utilisateur, **sans** trailer `Co-Authored-By`.
- Pas de `#[ignore]`-équivalent pour passer au vert : un test conditionnel n'est permis que pour le test d'intégration qui exige les modèles (tâche 7).

## Points de vigilance (Review Focus)

1. Micro refusé ou absent : la dictée ne démarre pas, l'état revient à `idle`, le champ n'est pas touché (tâche 4).
2. Téléchargement interrompu ou corrompu : état `failed`, aucun dossier de modèle partiel utilisable (tâche 5).
3. Champ fermé ou écran quitté pendant l'écoute : arrêt immédiat, aucune écriture dans un contrôleur détruit (tâches 4 et 9).
4. L'utilisateur tape ou déplace le curseur pendant la dictée : son texte n'est jamais écrasé (tâche 3).
5. Changement de langue ou durée maximale atteinte pendant l'écoute : arrêt propre, pas de fuite de micro (tâche 4).

## Structure des fichiers

| Fichier (sous `app/`) | Responsabilité |
|---|---|
| `lib/voice/dictation_types.dart` | langues, événements, problèmes, états, interfaces |
| `lib/voice/transcript_text.dart` | normalisation du texte reconnu |
| `lib/voice/dictation_inserter.dart` | écriture du partiel / du définitif dans un champ |
| `lib/voice/dictation_controller.dart` | machine d'états, une seule dictée à la fois |
| `lib/voice/voice_model_catalog.dart` | modèles pinnés (URL, taille, SHA-256) |
| `lib/voice/voice_model_store.dart` | téléchargement, vérification, extraction, suppression |
| `lib/voice/voice_preferences.dart` | langue choisie (SharedPreferences) |
| `lib/voice/sherpa_recognizer.dart`, `sherpa_isolate.dart` | reconnaissance en isolate |
| `lib/voice/pcm_convert.dart`, `record_audio_source.dart`, `parecord_audio_source.dart`, `audio_source_factory.dart` | capture micro |
| `lib/voice/voice_services.dart` | assemblage et accès par le contexte |
| `lib/ui/mic_button.dart` | bouton micro |
| `lib/ui/voice_settings.dart`, `voice_model_dialog.dart` | réglages et confirmation de téléchargement |

---

### Tâche 1 : socle — dépendances, types, interfaces

**Fichiers :** modifier `app/pubspec.yaml` ; créer `app/lib/voice/dictation_types.dart` ; tester `app/test/voice_types_test.dart`.

**Interfaces — produit (toutes les tâches suivantes s'appuient dessus) :**
```dart
enum DictationLanguage {
  fr('fr', 'Français'), en('en', 'English');
  const DictationLanguage(this.code, this.label);
  final String code; final String label;
  static DictationLanguage fromLocale(String localeCode); // 'en*' -> en, sinon fr
}
sealed class DictationEvent { const DictationEvent(this.text); final String text; }
final class DictationPartial extends DictationEvent { const DictationPartial(super.text); }
final class DictationFinal extends DictationEvent { const DictationFinal(super.text); }
enum DictationProblem { modelMissing, microphoneDenied, microphoneUnavailable, recognizerFailed }
class DictationException implements Exception { const DictationException(this.problem); final DictationProblem problem; }
enum DictationState { idle, preparing, listening, stopping }
abstract interface class AudioSource {
  Future<bool> requestPermission();
  Future<Stream<Float32List>> start();   // PCM mono 16 kHz, valeurs dans [-1, 1]
  Future<void> stop();
}
abstract interface class SpeechRecognizer {
  Stream<DictationEvent> get events;
  Future<void> start(DictationLanguage language);   // lève DictationException
  void feed(Float32List samples);
  Future<void> stop();                               // vide la fin, émet un DictationFinal, libère
}
abstract interface class ModelAvailability { bool isInstalled(DictationLanguage language); }
```

- [ ] **Étape 1 : test d'abord** — `voice_types_test.dart` : `fromLocale('fr_FR') == fr`, `fromLocale('en-GB') == en`, `fromLocale('de') == fr`, `DictationLanguage.en.code == 'en'`.
- [ ] **Étape 2 : lancer** `cd app && flutter test test/voice_types_test.dart` — attendu : échec (fichier absent).
- [ ] **Étape 3 :** ajouter à `pubspec.yaml` `sherpa_onnx: 2.0.1`, `record: ^7.2.0`, `archive: ^4.7.1`, `crypto: ^3.0.8` ; créer `dictation_types.dart` avec les interfaces ci-dessus ; `cd app && flutter pub get`.
- [ ] **Étape 4 : lancer** le test — attendu : PASS ; `cd app && flutter analyze lib/voice` — attendu : aucun problème.
- [ ] **Étape 5 : commit** `feat(voix): types et interfaces de la dictée, dépendances`.

### Tâche 2 : normalisation du texte reconnu

**Fichiers :** créer `app/lib/voice/transcript_text.dart` ; tester `app/test/voice_transcript_text_test.dart`.

**Produit :** `String normalizeTranscript(String raw)` — apostrophe `’` → `'`, espaces de tête et de fin retirés, espaces multiples réduits à un.

- [ ] **Étape 1 : test d'abord** — assertions : `normalizeTranscript(' Peux-tu m’envoyer  la facture ') == "Peux-tu m'envoyer la facture"` ; `normalizeTranscript('') == ''` ; `normalizeTranscript('   ') == ''` ; `normalizeTranscript('Hi, it’s me.') == "Hi, it's me."`.
- [ ] **Étape 2 : lancer** `flutter test test/voice_transcript_text_test.dart` — échec attendu.
- [ ] **Étape 3 :** implémenter `normalizeTranscript`. Aucune majuscule ni ponctuation ajoutée (le modèle les produit).
- [ ] **Étape 4 : lancer** — PASS.
- [ ] **Étape 5 : commit** `feat(voix): normalisation du texte reconnu`.

### Tâche 3 : insertion dans un champ (`DictationInserter`)

**Fichiers :** créer `app/lib/voice/dictation_inserter.dart` ; tester `app/test/voice_inserter_test.dart`.

**Consomme :** `TextEditingController` (le contrôleur de texte riche `RichController` en hérite). **Produit :**
```dart
class DictationInserter {
  DictationInserter(TextEditingController target);
  void showPartial(String text);   // remplace le partiel courant, ou l'insère au curseur
  void commit(String text);        // valide : texte + ' ', curseur après
  void release();                  // valide le partiel tel quel, ne touche plus au champ
}
```
Règles : le premier partiel s'ancre à la sélection (une sélection non vide est remplacée) ; une espace est ajoutée avant si le caractère précédent n'est pas un blanc ni un début de texte ; si le texte du champ ne correspond plus à ce que l'inserter a écrit (l'utilisateur a tapé ou déplacé le curseur), le partiel est conservé tel quel et le suivant s'ancre au nouveau curseur.

- [ ] **Étape 1 : tests d'abord** (`voice_inserter_test.dart`) :
  - `'le partiel est remplacé à chaque mise à jour'` : champ `'Bonjour '`, curseur 8 ; `showPartial('je')` → `'Bonjour je'` ; `showPartial('je vous')` → `'Bonjour je vous'` ; `commit('je vous confirme.')` → `'Bonjour je vous confirme. '`, curseur en fin.
  - `'espace ajoutée si le caractère précédent n'est pas un blanc'` : `'Bonjour'` → après `showPartial('je')` : `'Bonjour je'`.
  - `'insertion au milieu du texte'` : `'Salut  fin'`, curseur 6 ; `showPartial('toi')` → `c.text == 'Salut toi fin'` (les blancs existants autour du curseur ne sont pas doublés), et la sélection est juste après `toi`.
  - `'la frappe pendant un partiel est conservée'` : `'A '`, `showPartial('bc')` → `'A bc'` ; l'utilisateur fait `c.value = TextEditingValue(text: 'A bcX', selection: TextSelection.collapsed(offset: 5))` ; `showPartial('de')` → `'A bcX de'`.
  - `'release valide le partiel et ne touche plus au champ'` : après `release()`, `showPartial('zzz')` ne change pas `c.text`.
  - `'commit vide ne laisse pas d'espace parasite'` : `commit('')` retire le partiel sans rien ajouter.
- [ ] **Étape 2 : lancer** `flutter test test/voice_inserter_test.dart` — échec attendu.
- [ ] **Étape 3 :** implémenter ; garder la plage du partiel `[start, start+len)` et le dernier texte écrit pour détecter une modification extérieure.
- [ ] **Étape 4 : lancer** — PASS.
- [ ] **Étape 5 : commit** `feat(voix): insertion du texte dicté sans écraser la saisie`.

### Tâche 4 : contrôleur de dictée

**Fichiers :** créer `app/lib/voice/dictation_controller.dart` ; tester `app/test/voice_controller_test.dart` (avec des faux `AudioSource`, `SpeechRecognizer`, `ModelAvailability` définis dans `app/test/voice_fakes.dart`).

**Consomme :** types de la tâche 1, `DictationInserter`, `normalizeTranscript`. **Produit :**
```dart
class DictationController extends ChangeNotifier {
  DictationController({
    required AudioSource audio, required SpeechRecognizer recognizer,
    required ModelAvailability models,
    DictationLanguage language = DictationLanguage.fr,
    Duration maxDuration = const Duration(minutes: 5),
  });
  DictationState state; DictationProblem? problem;
  DictationLanguage language; TextEditingController? target;
  Future<void> start(TextEditingController target);   // arrête une dictée en cours sur une autre cible
  Future<void> stop();
  void release(TextEditingController target);          // la cible est détruite : arrêt sans écriture
  Future<void> setLanguage(DictationLanguage language); // arrête la dictée en cours
}
```
Chaque événement du reconnaisseur passe par `normalizeTranscript` puis l'inserter (`DictationPartial` → `showPartial`, `DictationFinal` → `commit`).

- [ ] **Étape 1 : tests d'abord** (`voice_controller_test.dart`, `fake_async` pour la durée) :
  - `'start passe par preparing puis listening ; les partiels écrivent dans le champ'`.
  - `'stop valide le dernier partiel et revient à idle'`.
  - `'micro refusé : problem == microphoneDenied, state == idle, champ intact'` (`requestPermission` → `false`).
  - `'micro indisponible : problem == microphoneUnavailable'` (`start()` du faux micro lève).
  - `'modèle absent : problem == modelMissing, le micro n'est pas ouvert'`.
  - `'une seconde cible arrête la première'` : `start(a)` puis `start(b)` → `target == b`, plus aucune écriture dans `a`.
  - `'release(target) arrête sans écrire'` : événement émis après `release` ne change pas le texte.
  - `'setLanguage pendant l'écoute arrête la dictée'` : `state == idle`, `language == en`.
  - `'durée maximale : idle après 5 minutes'` (`fakeAsync`, `elapse(Duration(minutes: 5, seconds: 1))`).
  - `'erreur du reconnaisseur : problem == recognizerFailed, micro arrêté'`.
- [ ] **Étape 2 : lancer** `flutter test test/voice_controller_test.dart` — échec attendu.
- [ ] **Étape 3 :** implémenter ; toute sortie (succès ou erreur) appelle `audio.stop()` et `recognizer.stop()` exactement une fois ; `notifyListeners()` à chaque changement d'état.
- [ ] **Étape 4 : lancer** — PASS.
- [ ] **Étape 5 : commit** `feat(voix): contrôleur de dictée, une dictée à la fois`.

### Tâche 5 : catalogue et magasin de modèles

**Fichiers :** créer `app/lib/voice/voice_model_catalog.dart`, `app/lib/voice/voice_model_store.dart` ; tester `app/test/voice_model_store_test.dart`.

**Produit :**
```dart
class VoiceModelEntry {
  const VoiceModelEntry({required this.language, required this.url, required this.archiveBytes,
      required this.sha256, required this.folder, required this.files});
  final DictationLanguage language; final Uri url; final int archiveBytes;
  final String sha256; final String folder; final List<String> files;
}
const List<VoiceModelEntry> voiceModelCatalog;   // fr et en : valeurs de la conception
sealed class VoiceModelStatus {}   // NotInstalled, Downloading(double progress), Installed, Failed(String reason)
class VoiceModelStore extends ChangeNotifier implements ModelAvailability {
  VoiceModelStore({required Directory root, List<VoiceModelEntry> catalog = voiceModelCatalog});
  VoiceModelStatus statusOf(DictationLanguage l);
  Future<void> refresh();                      // relit le disque
  Future<void> install(DictationLanguage l);   // télécharge, vérifie le SHA-256, extrait dans un isolate, renomme
  void cancel(DictationLanguage l);
  Future<void> remove(DictationLanguage l);
  Directory? directoryOf(DictationLanguage l); // null si non installé
}
```
`files` du catalogue : `encoder.onnx`, `decoder.onnx`, `joiner.onnx`, `tokens.txt`. Les URL, tailles et SHA-256 sont ceux du tableau « Modèles pinnés » de la conception.

- [ ] **Étape 1 : tests d'abord** — un `HttpServer` sur `127.0.0.1` sert une petite archive `tar.bz2` construite dans le test (`archive` : `TarEncoder` + `BZip2Encoder`) ; le catalogue du test pointe dessus avec sa vraie taille et son SHA-256 :
  - `'install réussie : Installed, directoryOf contient les 4 fichiers'`.
  - `'progression : Downloading monte de 0 à 1'`.
  - `'SHA-256 différent : Failed, directoryOf == null, aucun dossier résiduel'`.
  - `'connexion coupée en cours de route : Failed, aucun dossier partiel utilisable'`.
  - `'cancel pendant le téléchargement : NotInstalled, fichier temporaire supprimé'`.
  - `'remove supprime le dossier'` ; `'refresh détecte un modèle déjà présent'`.
  - `'archive incomplète (un fichier manquant) : Failed'`.
- [ ] **Étape 2 : lancer** `flutter test test/voice_model_store_test.dart` — échec attendu.
- [ ] **Étape 3 :** implémenter : téléchargement par `dart:io` `HttpClient` vers `<root>/<folder>.part`, empreinte par `crypto`, extraction dans `Isolate.run`, puis renommage atomique vers `<root>/<folder>`. Jamais de dossier final tant que tout n'est pas vérifié.
- [ ] **Étape 4 : lancer** — PASS.
- [ ] **Étape 5 : commit** `feat(voix): téléchargement vérifié des modèles de dictée`.

### Tâche 6 : préférences

**Fichiers :** créer `app/lib/voice/voice_preferences.dart` ; tester `app/test/voice_preferences_test.dart`.

**Produit :** `class VoicePreferences { VoicePreferences(SharedPreferences prefs); DictationLanguage language({required String uiLocale}); Future<void> setLanguage(DictationLanguage l); }`. Sans choix enregistré, `language(uiLocale: 'en_US') == en` et `'fr_FR'` → `fr` (via `fromLocale`).

- [ ] **Étape 1 : test d'abord** (`SharedPreferences.setMockInitialValues`) : défaut selon la locale ; après `setLanguage(en)`, `language(uiLocale: 'fr')` rend `en` ; valeur enregistrée inconnue → repli sur la locale.
- [ ] **Étape 2 : lancer** — échec attendu. **Étape 3 :** implémenter (clé `voice.language`). **Étape 4 : lancer** — PASS.
- [ ] **Étape 5 : commit** `feat(voix): mémorisation de la langue de dictée`.

### Tâche 7 : reconnaissance `sherpa_onnx` en isolate

**Fichiers :** créer `app/lib/voice/sherpa_recognizer.dart`, `app/lib/voice/sherpa_isolate.dart` ; tester `app/test/voice_sherpa_integration_test.dart`.

**Consomme :** `SpeechRecognizer`, `VoiceModelStore.directoryOf`. **Produit :** `class SherpaRecognizer implements SpeechRecognizer { SherpaRecognizer(VoiceModelStore store); }`.

Comportement (validé par le spike, voir la conception) : reconnaisseur `OnlineRecognizer` transducteur, `numThreads: 2`, `greedy_search`, fichiers `encoder.onnx` / `decoder.onnx` / `joiner.onnx` / `tokens.txt` du dossier du modèle ; `initBindings()` dans l'isolate ; un `DictationPartial` à chaque changement de texte ; à chaque fin de phrase (`isEndpoint`) un `DictationFinal` puis `reset` ; `stop()` ajoute 0,4 s de silence, appelle `inputFinished`, décode le reste, émet le dernier `DictationFinal`, libère le reconnaisseur et l'isolate. Modèle absent → `DictationException(modelMissing)`. Erreur native → `DictationException(recognizerFailed)`.

- [ ] **Étape 1 : test d'intégration d'abord**, exécuté seulement si la variable `MIRA_VOICE_MODELS` pointe un dossier contenant les deux modèles décompressés (sinon `skip: 'modèles absents (MIRA_VOICE_MODELS)'`). Le test copie les modèles dans un `VoiceModelStore` temporaire, lit `test_wavs/0.wav` du modèle, le passe par `feed` par paquets de 1 600 échantillons, puis `stop()` :
  - français : le texte final concaténé contient `pays` et `demandez` ;
  - anglais : il contient `country` ;
  - `'stop vide la fin'` : le dernier mot du wav (`faire`/`do`) est présent après `stop()`.
- [ ] **Étape 2 : lancer** `MIRA_VOICE_MODELS=<dossier des modèles> flutter test test/voice_sherpa_integration_test.dart` — échec attendu. Si la bibliothèque native n'est pas résolue sous `flutter test`, déplacer le fichier dans `app/integration_test/` et le lancer avec `flutter test integration_test -d linux`.
- [ ] **Étape 3 :** implémenter ; l'isolate reçoit des `TransferableTypedData`, jamais de gros messages copiés.
- [ ] **Étape 4 : lancer** — PASS ; sans la variable, le test est signalé « skipped » et non en échec.
- [ ] **Étape 5 : commit** `feat(voix): reconnaissance en flux sherpa-onnx dans un isolate`.

### Tâche 8 : capture du micro

**Fichiers :** créer `app/lib/voice/pcm_convert.dart`, `app/lib/voice/record_audio_source.dart`, `app/lib/voice/parecord_audio_source.dart`, `app/lib/voice/audio_source_factory.dart` ; tester `app/test/voice_pcm_test.dart`.

**Produit :**
```dart
class Pcm16Decoder { Float32List add(Uint8List chunk); }   // garde l'octet impair entre deux paquets
AudioSource audioSourceForPlatform();
```

- [ ] **Étape 1 : test d'abord** (`voice_pcm_test.dart`) : `[0x00, 0x40]` → `[0.5]` ; `[0x00, 0x80]` → `[-1.0]` ; paquet de 3 octets puis paquet de 1 octet → 2 échantillons au total, jamais de perte ni de décalage ; paquet vide → liste vide.
- [ ] **Étape 2 : lancer** — échec attendu. **Étape 3 :** implémenter `Pcm16Decoder`.
- [ ] **Étape 4 : lancer** — PASS.
- [ ] **Étape 5 :** `RecordAudioSource` : `AudioRecorder().startStream(RecordConfig(encoder: AudioEncoder.pcm16bits, sampleRate: 16000, numChannels: 1))` ; `requestPermission` → `hasPermission()` ; flux converti par `Pcm16Decoder`. `ParecordAudioSource` : processus `parecord --raw --format=s16le --rate=16000 --channels=1 --latency-msec=100` (commande validée par le spike), tué dans `stop()`. `audioSourceForPlatform()` : Linux → d'abord vérifier à la main (étape 6) que `record` fonctionne ; sinon `ParecordAudioSource` ; ailleurs → `RecordAudioSource`.
- [ ] **Étape 6 : vérification manuelle sous Linux** — petit `main` jetable (hors dépôt, dans le scratchpad) qui ouvre `audioSourceForPlatform()` 3 secondes et affiche le nombre d'échantillons reçus. Attendu : environ 48 000. Consigner le résultat et le choix dans `docs/VOICE.md` (tâche 11).
- [ ] **Étape 7 : commit** `feat(voix): capture du micro (record, repli parecord sous Linux)`.

### Tâche 9 : bouton micro et intégration chat + composeur

**Fichiers :** créer `app/lib/voice/voice_services.dart`, `app/lib/ui/mic_button.dart` ; modifier `app/lib/main.dart` (fournir les services), `app/lib/ui/assistant_panel.dart` (près de la ligne 418, champ du chat), `app/lib/ui/composer.dart` (`_FmtBar` près de la ligne 678, cible : `Store.body`, le `RichController` du corps, `app/lib/state/store.dart:72`) ; tester `app/test/voice_mic_button_test.dart`.

**Produit :**
```dart
class VoiceServices { final DictationController dictation; final VoiceModelStore models; final VoicePreferences preferences; }
VoiceServices? voiceServicesOf(BuildContext context);   // null si non fourni
class MicButton extends StatefulWidget { const MicButton({super.key, required this.target, this.size = 28}); final TextEditingController target; final double size; }
```
`MicButton` rend `SizedBox.shrink()` si `voiceServicesOf(context)` est `null` (ainsi les tests existants de l'app ne changent pas). Clés : `voice-mic-button`, `voice-mic-listening`. Au `dispose`, appelle `release(target)`. Si `problem == modelMissing`, ouvre la boîte de confirmation de la tâche 10 ; si `microphoneDenied` / `microphoneUnavailable`, affiche un message par `t()`.

- [ ] **Étape 1 : tests d'abord** (faux services de `voice_fakes.dart`) :
  - `'sans services : aucun bouton'` ;
  - `'toucher démarre la dictée sur le champ, retoucher l'arrête'` ;
  - `'état écoute : voice-mic-listening visible'` ;
  - `'retirer le widget pendant l'écoute libère la cible'` (`release` appelé, plus d'écriture) ;
  - `'micro refusé : message affiché'`.
- [ ] **Étape 2 : lancer** `flutter test test/voice_mic_button_test.dart` — échec attendu.
- [ ] **Étape 3 :** implémenter ; brancher dans `main.dart` (`MultiProvider` autour de `MiraApp`, ou un `InheritedWidget` dédié lu par `voiceServicesOf`) ; placer le bouton dans la rangée d'outils du chat et dans la barre du composeur.
- [ ] **Étape 4 : lancer** le test et les tests existants touchés : `flutter test test/voice_mic_button_test.dart test/assistant_panel_ai_test.dart` et les tests du composeur existants (`ls app/test | grep -i compos`) — PASS.
- [ ] **Étape 5 :** `python3 tools/i18n-audit` — attendu : 0 ; ajouter les textes manquants aux fichiers de langue.
- [ ] **Étape 6 : commit** `feat(voix): bouton micro dans le chat et le composeur`.

### Tâche 10 : réglages, confirmation de téléchargement, attribution

**Fichiers :** créer `app/lib/ui/voice_settings.dart`, `app/lib/ui/voice_model_dialog.dart` ; modifier `app/lib/ui/settings_panel.dart` (nouvelle section « Dictée vocale »), `app/lib/ui/about_settings.dart` (attribution) ; tester `app/test/voice_settings_test.dart`.

Contenu : sélecteur de langue FR / EN ; une carte par langue (nom, 57 Mo, état, boutons Télécharger / Annuler / Supprimer, barre de progression) ; avant tout téléchargement, une boîte dit la taille, la source (`github.com/k2-fsa/sherpa-onnx`), que rien d'autre n'est envoyé et que la voix reste sur la machine ; mention des limites (noms propres rares) ; attribution « modèles Kroko, licence CC-BY-SA » dans « À propos ».

- [ ] **Étape 1 : tests d'abord** : `'télécharger demande confirmation et ne lance rien avant'` ; `'refuser la confirmation n'appelle pas install'` ; `'confirmer appelle install(langue)'` ; `'changer de langue enregistre et appelle setLanguage'` ; `'état Failed affiche la raison et permet de réessayer'`.
- [ ] **Étape 2 : lancer** `flutter test test/voice_settings_test.dart` — échec attendu.
- [ ] **Étape 3 :** implémenter ; réutiliser le style et les composants de `ai_local_panel.dart` / `ai_local_confirm.dart` (même modèle : carte + confirmation).
- [ ] **Étape 4 : confirmer la licence** — lire le fichier `LICENSE` du dépôt `Banafo/Kroko-ASR` et la fiche des deux archives pinnées ; si elles ne sont pas sous CC-BY-SA, **s'arrêter et le signaler** à l'utilisateur avant de continuer. Écrire la version exacte de la licence dans l'attribution.
- [ ] **Étape 5 : lancer** le test ; `python3 tools/i18n-audit` — 0.
- [ ] **Étape 6 : commit** `feat(voix): réglages de la dictée, confirmation de téléchargement, attribution`.

### Tâche 11 : plateformes, documentation, nettoyage

**Fichiers :** modifier `app/android/app/src/main/AndroidManifest.xml` ; créer `docs/VOICE.md` ; supprimer `spike/`.

- [ ] **Étape 1 : Android** — ajouter `<uses-permission android:name="android.permission.RECORD_AUDIO"/>` ; vérifier que `flutter build apk --debug` aboutit (réseau Gradle nécessaire : si indisponible, le dire et le laisser à l'utilisateur) ; noter l'augmentation de taille de l'APK.
- [ ] **Étape 2 : Windows** — ne peut pas être testé depuis cette machine. Écrire dans `docs/VOICE.md` un **protocole manuel** : installer, télécharger le modèle FR et EN, dicter la phrase de test du plan (« Bonjour Marie, je vous confirme notre rendez-vous de jeudi à quatorze heures »), vérifier le micro, l'arrêt, la fermeture du champ. Même protocole pour Android (appareil modeste : relever fluidité et chauffe).
- [ ] **Étape 3 : `docs/VOICE.md`** — architecture, vie privée, licence, choix `record` / `parecord`, résultats du spike, limites connues (noms propres, termes mixtes), comment ajouter une langue (une ligne dans le catalogue, un modèle, un libellé).
- [ ] **Étape 4 : nettoyage** — supprimer `spike/` (jetable ; les résultats sont dans la conception et `docs/VOICE.md`), sauf demande contraire de l'utilisateur de le garder comme `tools/voice-probe`.
- [ ] **Étape 5 :** tests ciblés de tout le module : `flutter test test/voice_*_test.dart` — PASS (le test d'intégration signalé « skipped » sans modèles) ; `flutter analyze` — aucun problème nouveau.
- [ ] **Étape 6 :** dire que la CI complète n'a pas tourné et la proposer ; ne rien lancer sans accord.
- [ ] **Étape 7 : commit** `docs(voix): protocole de test par plateforme, guide de la dictée` (et `feat(voix): permission micro Android`).

## Ordre et parallélisme

1 → (2, 3, 6 en parallèle : fichiers disjoints) → 4 → 5 → (7, 8 en parallèle) → 9 → 10 → 11.

## Auto-vérification

- **Couverture de la conception :** dictée chat et brouillon (9) ; local (5, 7) ; FR/EN à égalité (1, 5, 6, 10) ; portable Linux/Windows/Android (8, 11) ; texte utilisable (2) ; une dictée à la fois, arrêts (4) ; téléchargement confirmé (10) ; licence (10, 11) ; isolate (7) ; aucune modification du moteur ni du pont.
- **Hors plan, assumé :** mots favorisés, macOS (pas de plateforme `macos/` dans `app/` aujourd'hui), mode mains libres.
