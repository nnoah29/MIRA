import 'dart:io';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

/// Nom du fichier de la bibliothèque du moteur sur la plateforme donnée.
String bridgeLibraryFileName(String os) =>
    os == 'windows' ? 'mira_bridge.dll' : 'libmira_bridge.so';

/// Chemins où chercher la bibliothèque, du plus explicite au plus courant.
List<String> bridgeLibraryPaths({
  required String os,
  required String executableDir,
  required String? override,
}) {
  final name = bridgeLibraryFileName(os);
  final bundled = os == 'windows'
      ? '$executableDir/$name'
      : '$executableDir/lib/$name';
  return [
    ?override,
    bundled,
    '../bridge/target/release/$name',
    '../bridge/target/debug/$name',
    name,
  ];
}

/// Chemins où chercher la bibliothèque du moteur sur cette machine.
List<String> bridgeLibraryCandidates() => bridgeLibraryPaths(
  os: Platform.operatingSystem,
  executableDir: File(Platform.resolvedExecutable).parent.path,
  override: Platform.environment['MIRA_BRIDGE_LIB'],
);

/// Charge la bibliothèque du moteur (première candidate qui existe, sinon le chargeur système).
ExternalLibrary openBridgeLibrary() {
  final found = bridgeLibraryCandidates().where((p) => File(p).existsSync());
  return ExternalLibrary.open(
    found.isEmpty
        ? bridgeLibraryFileName(Platform.operatingSystem)
        : found.first,
  );
}
