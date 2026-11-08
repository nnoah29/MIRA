# Windows 5b-1 : démon sous Windows, plan

**Spec :** `docs/superpowers/specs/2026-11-08-windows-phase-5b-demon-design.md`. Exécution native, tests ciblés, un commit par tâche, jamais `./ci.sh` sans accord.

**Contraintes :** normes `tools/check-norms` ; pas d'`unsafe` ; pas d'`unwrap`/`expect` hors tests.

## Tâche 1 : le démon compile hors Linux
- `Cargo.toml` : `zbus` et `ksni` sous `[target.'cfg(target_os = "linux")'.dependencies]`.
- `lib.rs` : `secret_service` seulement sous Linux ; nouveau `probe.rs` avec `probe_for(file_secrets)` (Linux : `SecretServiceProbe` ; ailleurs : `NeverLocked`) ; `run.rs` l'utilise.
- `desktop/` : `hints`, `launch_app`, `proxy`, `sink`, `state_mirror`, `tray` et leurs exports sous Linux ; `Desktop::connect` séparé par système ; le champ `_tray` devient `Option<Arc<dyn Debug + Send + Sync>>`.
- Tests Linux seulement (`#![cfg(target_os = "linux")]`) : ceux qui touchent D-Bus, socket Unix, `XDG_*`.
- Vérif : `cargo check -p mira-daemon` et un test ciblé existant (`--test token`).

## Tâche 2 : fichier de port, TCP par défaut sous Windows
- `DaemonPaths.port` (`<profil>/daemon.port`) ; `transport/port_file.rs` : `PortFile::publish(path, port)` (ouverture partagée en lecture sous Windows, `AddrInUse` si pris), `Drop` supprime.
- `listen.rs` : TCP lié si `tcp_port` ou Windows ; publication du port ; échec d'instance unique code 3.
- Test `tests/port_file.rs` : publier, relire, second refusé, supprimé au drop.

## Tâche 3 : app Dart
- `platform_info.dart` : `canUseDaemon`. `daemon_paths.dart` : racine `LOCALAPPDATA\mira\profiles` et `port`. `endpoint`/`engine_opening` : sur Windows lire le port et ouvrir un `DaemonEndpoint` TCP. `daemon_launcher.dart` : `mira-daemon.exe`, sortie vidée.
- Tests Dart ciblés pour chaque fonction ajoutée ; `flutter analyze` propre.

## Tâche 4 : paquet et CI
- `packaging/windows/build.ps1` construit `mira-daemon` ; `app/windows/CMakeLists.txt` installe `mira-daemon.exe` ; `windows.yml` : `cargo build -p mira-daemon` + `cargo test -p mira-daemon --test port_file`.
- `docs/ARCHITECTURE-IMPLEMENTEE.md` : section démon Windows.
