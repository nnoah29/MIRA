# Dictée vocale

Dicter dans le corps d'un brouillon et dans le champ du chat avec l'IA. Les mots apparaissent pendant
qu'on parle. La reconnaissance est **locale** : la voix et le texte dicté ne quittent jamais la machine.
Conception : `docs/superpowers/specs/2026-11-15-dictee-vocale-design.md`.

## Architecture (tout est dans l'app Flutter, le moteur Rust ne change pas)

```
micro (record) ─▶ Pcm16Decoder ─▶ SherpaRecognizer (isolate, sherpa-onnx, modèle Kroko)
                                         │ DictationPartial / DictationFinal
                                         ▼
                 DictationController ─▶ normalizeTranscript ─▶ DictationInserter ─▶ champ de texte
```

| Fichier (`app/lib/voice/`) | Rôle |
|---|---|
| `dictation_types.dart` | langues, événements, problèmes, états, interfaces `AudioSource` / `SpeechRecognizer` / `ModelAvailability` |
| `dictation_controller.dart` | une dictée à la fois pour toute l'app, arrêt à la fermeture du champ, au changement de langue, après 5 min |
| `dictation_inserter.dart` | partiel remplacé en direct, définitif validé, la frappe de l'utilisateur n'est jamais écrasée |
| `sherpa_recognizer.dart`, `sherpa_isolate.dart` | reconnaissance en flux dans un isolate (l'interface ne fige jamais) |
| `record_audio_source.dart`, `pcm_convert.dart` | micro PCM 16 bits 16 kHz converti en flottants |
| `voice_model_catalog.dart`, `voice_model_store.dart`, `voice_model_archive.dart` | modèles pinnés (SHA-256), téléchargement vérifié, extraction, suppression |
| `voice_preferences.dart`, `voice_services.dart` | langue mémorisée, assemblage et accès par le contexte |

Interface : `ui/mic_button.dart` (chat et composeur), `ui/voice_settings.dart` (onglet Voix),
`ui/voice_model_dialog.dart` (confirmation du téléchargement). Sans `VoiceScope`, aucun bouton n'apparaît.

## Vie privée

- Le seul accès réseau de la dictée est le téléchargement du modèle, après confirmation explicite
  (taille et source affichées). Aucune voix ni texte dicté n'est envoyé.
- Les modèles sont dans le dossier de support de l'app (`voice-models/`) et se suppriment depuis les réglages.

## Licence des modèles : à vérifier avant toute publication

Les modèles **Kroko communautaires** (Banafo) sont annoncés sous **CC-BY-SA** par l'éditeur, sans version
précisée ; le fichier `LICENSE` du dépôt Hugging Face est vide et la fiche indique « other ». MIRA ne les
redistribue pas (téléchargement par l'utilisateur depuis `github.com/k2-fsa/sherpa-onnx`), et l'attribution
figure dans « À propos ». Une confirmation de l'éditeur est à obtenir avant une release.

## Résultats du spike (Linux, voix de l'auteur, 15 novembre 2026)

- Kroko FR et EN (57 Mo chacun) : direct fluide, mises à jour d'environ 1,2 s, décodage 40–350 ms par paquet,
  ponctuation et casse produites par le modèle.
- Modèles de référence plus gros (FR 398 Mo, EN 128 Mo) : moins précis, sans ponctuation, en majuscules. Écartés.
- Limites : noms propres rares, sigles épelés, termes anglais dans une phrase française, accent non natif.
- Mots favorisés (contacts) : non démontré sans le fichier SentencePiece de Kroko. Reporté.

## Micro sous Linux

`record_linux` s'appuie sur `parecord` (PulseAudio ou PipeWire-pulse). Vérifié sur le vrai micro :
3 s de capture donnent environ 45 000 échantillons. Un système sans `parecord` obtient « Micro introuvable ».

## Lancer les tests

```
cd app
flutter test test/voice_*_test.dart                      # tout sauf l'intégration native
LD_LIBRARY_PATH=$HOME/.pub-cache/hosted/pub.dev/sherpa_onnx_linux-2.0.1/linux/x64 \
MIRA_VOICE_MODELS=<dossier des modèles décompressés> \
  flutter test test/voice_sherpa_integration_test.dart   # nécessite les modèles
```

Sans `MIRA_VOICE_MODELS`, les deux tests d'intégration sont signalés « skipped ».
Sous `flutter test`, la bibliothèque native n'est pas résolue d'elle-même : d'où `LD_LIBRARY_PATH`.

## Ajouter une langue

Un modèle Kroko supplémentaire : une valeur dans `DictationLanguage`, une entrée dans
`voice_model_catalog.dart` (URL, taille, SHA-256 de l'archive), les textes dans les fichiers de traduction.

## Statut par plateforme

| Plateforme | Statut |
|---|---|
| Linux | validé : micro réel, reconnaissance FR et EN, tests automatisés |
| Windows | **non testé** (bibliothèques fournies par `sherpa_onnx_windows` et `record_windows`) ; options natives étudiées et écartées, voir `docs/VOICE-WINDOWS.md` |
| Android | permission `RECORD_AUDIO` ajoutée ; **non construit ni testé** (licences du SDK non acceptées sur la machine de développement) |
| macOS | pas de plateforme `macos/` dans `app/` ; le code est neutre |

## Protocole manuel (Windows, Android)

1. Installer, ouvrir Réglages › Voix, télécharger le modèle français puis l'anglais.
2. Ouvrir un brouillon, toucher le micro, dire : « Bonjour Marie, je vous confirme notre rendez-vous de jeudi
   à quatorze heures. » Les mots doivent apparaître en direct, avec ponctuation.
3. Retoucher le micro : la dictée s'arrête, le texte reste. Fermer le brouillon en dictant : plus d'écriture.
4. Refuser l'autorisation du micro : un message l'explique. Supprimer le modèle : le micro propose de le retélécharger.
5. Android : sur un appareil modeste, relever la fluidité et l'échauffement ; mesurer le poids de l'APK.
