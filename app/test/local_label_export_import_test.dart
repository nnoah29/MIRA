import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/local_label_admin.dart';

/// Moteur factice : garde les requêtes et rend la réponse programmée du domaine `local_label`.
class _Engine implements EngineClient {
  final sent = <Map<String, dynamic>>[];
  Map<String, dynamic> answer = {'type': 'done'};

  Map<String, dynamic> get last => sent.last['request'] as Map<String, dynamic>;

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async {
    sent.add(request);
    return {'type': 'local_label', 'data': answer};
  }

  @override
  Stream<Map<String, dynamic>> events({int? after}) => const Stream.empty();

  @override
  Future<String?> originalHtml(String messageId) async => null;

  @override
  Future<void> close() async {}
}

void main() {
  test(
    'exportLabelsFile envoie export_local_labels_file et rend le texte',
    () async {
      final engine = _Engine()
        ..answer = {
          'type': 'labels_file_export',
          'data': {'yaml': 'mira_export: {}\n'},
        };
      final yaml = await LocalLabelAdmin(engine).exportLabelsFile();
      expect(engine.sent.single['domain'], 'local_label');
      expect(engine.last['op'], 'export_local_labels_file');
      expect(yaml, 'mira_export: {}\n');
    },
  );

  test('importLabelsFile envoie le texte, l’option et une clé neuve à chaque appel', () async {
    final engine = _Engine()
      ..answer = {
        'type': 'labels_file_import',
        'data': {'entries': []},
      };
    final admin = LocalLabelAdmin(engine);
    await admin.importLabelsFile('a: 1', replaceExisting: true);
    final first = engine.last;
    await admin.importLabelsFile('a: 1');
    final second = engine.last;
    expect(first['op'], 'import_local_labels_file');
    expect(first['yaml'], 'a: 1');
    expect(first['replace_existing'], true);
    expect(second['replace_existing'], false);
    expect(first['idempotency_key'], isNotEmpty);
    expect(first['idempotency_key'], isNot(second['idempotency_key']));
  });

  test(
    'le rapport décode les compteurs et un refus avec sa position et son nom',
    () async {
      final engine = _Engine()
        ..answer = {
          'type': 'labels_file_import',
          'data': {
            'entries': [
              {
                'position': 1,
                'name': 'Compta',
                'outcome': {'type': 'imported'},
              },
              {
                'position': 2,
                'name': 'Rouge',
                'outcome': {
                  'type': 'refused',
                  'reason': {'type': 'invalid_color'},
                },
              },
            ],
          },
        };
      final report = await LocalLabelAdmin(engine).importLabelsFile('x');
      expect(report.imported, 1);
      expect(report.refused, 1);
      expect(report.entries[1].position, 2);
      expect(report.entries[1].name, 'Rouge');
      expect(report.entries[1].reason, 'invalid_color');
    },
  );
}
