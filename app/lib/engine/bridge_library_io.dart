import 'dart:io';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

/// Chemins où chercher la bibliothèque du moteur, du plus explicite au plus courant.
List<String> bridgeLibraryCandidates() => [
  ?Platform.environment['MIRA_BRIDGE_LIB'],
  '${File(Platform.resolvedExecutable).parent.path}/lib/libmira_bridge.so',
  '../bridge/target/release/libmira_bridge.so',
  '../bridge/target/debug/libmira_bridge.so',
  'libmira_bridge.so',
];

/// Charge la bibliothèque du moteur (première candidate qui existe, sinon le chargeur système).
ExternalLibrary openBridgeLibrary() {
  final found = bridgeLibraryCandidates().where((p) => File(p).existsSync());
  return ExternalLibrary.open(
    found.isEmpty ? 'libmira_bridge.so' : found.first,
  );
}
