@TestOn('linux')
library;

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/engine_mail_source.dart';
import 'package:mira/engine/rust_engine_client.dart';

/// Vrai moteur Rust par FFI, sur un profil jetable (`MIRA_HOME`) ; ignoré sans bibliothèque compilée.
void main() {
  final lib = bridgeLibraryCandidates().where((p) => File(p).existsSync());
  final skip = lib.isEmpty
      ? 'bibliothèque absente (cd bridge && cargo build --release)'
      : Platform.environment['MIRA_HOME'] == null
      ? 'lancer avec MIRA_HOME=<dossier jetable> pour ne pas toucher au vrai profil'
      : null;

  test('le moteur répond au contrat par FFI', () async {
    final client = await RustEngineClient.open();
    final info = await client.call(req('engine', 'get_contract_info'));
    expect(info['type'], 'contract_info');
    expect((info['data'] as Map)['version'], {'major': 1, 'minor': 2});
    final source = await EngineMailSource.load(client);
    expect(source.accounts, isEmpty);
    final st = await source.maintenance.stats();
    expect(st.messages, 0);
    expect(await source.maintenance.checkIntegrity(), isTrue);
    expect(await source.admin.syncStatus(), isEmpty);
    expect(await source.filtered('aucun', 'starred', unread: true), isEmpty);
    await source.dispose();
    await client.close();
  }, skip: skip);
}
