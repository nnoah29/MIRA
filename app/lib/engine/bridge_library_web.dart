import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

/// Aucune bibliothèque native dans un navigateur : la démonstration n'ouvre pas de moteur.
List<String> bridgeLibraryCandidates() => const [];

/// Le moteur Rust n'existe pas dans un navigateur.
ExternalLibrary openBridgeLibrary() => throw UnsupportedError(
  'moteur indisponible dans un navigateur',
); // i18n:ignore
