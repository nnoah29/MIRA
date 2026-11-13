import 'package:flutter_test/flutter_test.dart';
import 'package:mira/i18n/engine_errors.dart';
import 'package:mira/engine/engine_client.dart';

import 'ai_settings_harness.dart';
import 'fake_ai_engine.dart';

void main() {
  testWidgets(
    'utiliser comme assistant règle le modèle puis la connexion par défaut',
    (tester) async {
      final engine = FakeAiEngine()
        ..providers.add(provider('local', 'Modèle local', local: true))
        ..connections.add(connection('loc-1', provider: 'local'));
      engine.local.installed.add('mini');
      await pumpSettings(tester, using: engine);
      await tapKey(tester, 'ai-tab-local');
      await tapKey(tester, 'ai-local-use-mini');
      expect(
        engine.requestsOf('update_ai_connection').single['default_model'],
        'mini',
      );
      expect(
        engine.requestsOf('set_default_ai_connection').single['connection'],
        'loc-1',
      );
    },
  );

  testWidgets('un échec d’installation affiche le texte du code du moteur', (
    tester,
  ) async {
    final engine = FakeAiEngine()
      ..providers.add(provider('local', 'Modèle local', local: true));
    engine.local.failInstall = const EngineException(
      'incompatible_state.ai_local_disk_space',
      '',
    );
    await pumpSettings(tester, using: engine);
    await tapKey(tester, 'ai-tab-local');
    await tapKey(tester, 'ai-local-install-mini');
    await tapKey(tester, 'ai-local-confirm-accept');
    expect(find.textContaining('Pas assez d’espace disque'), findsOneWidget);
  });

  test('chaque code d’erreur du modèle local a un texte', () {
    for (final code in [
      'incompatible_state.ai_local_busy',
      'incompatible_state.ai_local_disk_space',
      'incompatible_state.ai_local_unsupported_platform',
      'server.ai_local_checksum_mismatch',
      'server.ai_local_runtime_failed',
      'server.ai_local_start_timeout',
    ]) {
      expect(engineErrorTexts[code], isNotNull, reason: code);
      expect(
        engineErrorText(EngineException(code, '')),
        isNot(contains('Erreur :')),
      );
    }
  });
}
