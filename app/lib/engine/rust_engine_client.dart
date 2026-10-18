import 'dart:convert';
import 'dart:io';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

import '../src/rust/api/engine.dart' as ffi;
import '../src/rust/frb_generated.dart';
import 'engine_client.dart';

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

/// Client du moteur Rust par flutter_rust_bridge ; un seul par processus.
class RustEngineClient implements EngineClient {
  RustEngineClient._();

  /// Charge la bibliothèque, vérifie le contrat et ouvre le profil.
  static Future<RustEngineClient> open({
    String profile = 'default',
    int? pollSecs,
  }) async {
    await RustLib.init(externalLibrary: openBridgeLibrary());
    openEnvelope(await ffi.openEngine(profile: profile, pollSecs: pollSecs));
    return RustEngineClient._();
  }

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async =>
      openEnvelope(await ffi.execute(requestJson: jsonEncode(request)));

  @override
  Stream<Map<String, dynamic>> events({int? after}) => ffi
      .watchEvents(after: after == null ? null : BigInt.from(after))
      .map((line) => jsonDecode(line) as Map<String, dynamic>);

  @override
  Future<String?> originalHtml(String messageId) async =>
      openEnvelope(await ffi.getOriginalHtml(message: messageId))['html']
          as String?;

  @override
  Future<void> close() async {
    openEnvelope(await ffi.closeEngine());
  }
}
