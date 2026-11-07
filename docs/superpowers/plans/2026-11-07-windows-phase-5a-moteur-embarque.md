# Windows 5a, moteur embarqué : plan

**Goal :** l'app MIRA compile et s'exécute sous Windows avec le moteur embarqué.
**Spec :** `docs/superpowers/specs/2026-11-07-windows-phase-5a-moteur-embarque-design.md`
**Exécution :** native, TDD, un commit par tâche, aucun `./ci.sh` ni suite complète.

## Tâches
1. **Moteur portable** : `ProfileEnvironment::from_lookup` (repli `LOCALAPPDATA`), `cfg(unix)` pour droits des dossiers/fichiers dans `profile/paths.rs`, `maintenance/export_file.rs`, `mira-host/.../sealed_file.rs`. Tests : `profile_environment` (repli Windows), tests existants de profil/export/trousseau.
2. **Dart plateforme** : `runsOnWindows`, `bridgeLibraryNames`/candidats par plateforme (`mira_bridge.dll`), initialisation Windows des notifications. Tests : `bridge_library_test`.
3. **Cible Windows** : `app/windows` (flutter create), CMake qui installe `mira_bridge.dll`, ignore des artefacts.
4. **Vérification et docs** : workflow `.github/workflows/windows.yml` (cargo check + build du pont + `flutter build windows`), AG-93 et `ARCHITECTURE-IMPLEMENTEE.md`.
