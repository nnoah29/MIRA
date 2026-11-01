import 'dart:convert';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

import '../src/rust/api/engine.dart' as ffi;
import '../src/rust/frb_generated.dart';
import 'bridge_library.dart';
import 'engine_client.dart';

export 'bridge_library.dart' show bridgeLibraryCandidates;

/// Client du moteur Rust par flutter_rust_bridge ; un seul par processus.
class RustEngineClient implements EngineClient, ChatStreamingClient {
  RustEngineClient._();

  /// Charge la bibliothèque, vérifie le contrat et ouvre le profil.
  /// `home` et `secretsKey` remplacent le dossier de données et le trousseau du système (Android).
  static Future<RustEngineClient> open({
    String profile = 'default',
    String? home,
    Uint8List? secretsKey,
  }) async {
    await RustLib.init(externalLibrary: openBridgeLibrary());
    openEnvelope(
      await ffi.openEngine(
        profile: profile,
        home: home,
        secretsKey: secretsKey,
      ),
    );
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
  Stream<Map<String, dynamic>> chatStream(Map<String, dynamic> start) => ffi
      .chatStream(requestJson: jsonEncode(start))
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
