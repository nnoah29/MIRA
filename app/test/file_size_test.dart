import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/ui/rules_file_flow.dart';

void main() {
  test('un fichier de 2 Mo au plus est accepté, même de taille inconnue avant lecture', () {
    expect(() => ensureRulesFileSize(0), returnsNormally);
    expect(() => ensureRulesFileSize(maxRulesFileBytes), returnsNormally);
  });

  test('au-delà de 2 Mo, la lecture est refusée avec le code du moteur', () {
    expect(
      () => ensureRulesFileSize(maxRulesFileBytes + 1),
      throwsA(
        isA<EngineException>().having(
          (e) => e.code,
          'code',
          'invalid_input.export_too_large',
        ),
      ),
    );
  });

  test('les noms de fichiers sont datés', () {
    final day = DateTime(2026, 10, 27);
    expect(rulesExportName(day), 'mira-regles-2026-10-27.yaml');
    expect(labelsExportName(day), 'mira-libelles-2026-10-27.yaml');
  });
}
