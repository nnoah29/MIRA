import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/rule_admin.dart';

/// Moteur factice : enregistre les requêtes et rend la réponse programmée.
class _Engine implements EngineClient {
  final calls = <Map<String, dynamic>>[];
  final Map<String, dynamic> answer;
  _Engine(this.answer);

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async {
    calls.add(request);
    return answer;
  }

  @override
  Stream<Map<String, dynamic>> events({int? after}) => const Stream.empty();

  @override
  Future<String?> originalHtml(String messageId) async => null;

  @override
  Future<void> close() async {}
}

Map<String, dynamic> _report() => {
  'type': 'rule',
  'data': {
    'type': 'rules_import',
    'data': {
      'entries': [
        {
          'position': 1,
          'name': 'Lettres',
          'outcome': {'type': 'imported'},
        },
        {
          'position': 2,
          'name': 'Cassée',
          'outcome': {
            'type': 'refused',
            'reason': {'type': 'unknown_label'},
          },
        },
        {
          'position': 3,
          'name': 'Vide',
          'outcome': {
            'type': 'refused',
            'reason': {
              'type': 'invalid_rule',
              'code': 'invalid_input.parameter',
            },
          },
        },
        {
          'position': 4,
          'name': 'Là',
          'outcome': {'type': 'already_present'},
        },
      ],
    },
  },
};

void main() {
  test('exportRules envoie export_rules et rend le texte du moteur', () async {
    final engine = _Engine({
      'type': 'rule',
      'data': {
        'type': 'rules_export',
        'data': {
          'yaml': 'mira_export: {}\n',
          'skipped': ['Factures'],
        },
      },
    });
    final export = await RuleAdmin(engine).exportRules();
    final request = engine.calls.single;
    expect(request['domain'], 'rule');
    expect((request['request'] as Map)['op'], 'export_rules');
    expect(export.yaml, 'mira_export: {}\n');
    expect(export.skipped, ['Factures']);
  });

  test(
    'importRules envoie le texte, l’option et une clé neuve à chaque appel',
    () async {
      final engine = _Engine(_report());
      final admin = RuleAdmin(engine);
      await admin.importRules('a: 1', replaceExisting: true);
      await admin.importRules('a: 1', replaceExisting: false);
      final first = engine.calls[0]['request'] as Map;
      final second = engine.calls[1]['request'] as Map;
      expect(first['op'], 'import_rules');
      expect(first['yaml'], 'a: 1');
      expect(first['replace_existing'], true);
      expect(second['replace_existing'], false);
      expect(first['idempotency_key'], isNotEmpty);
      expect(first['idempotency_key'], isNot(second['idempotency_key']));
    },
  );

  test('le rapport décode les compteurs, les refus et leurs raisons', () async {
    final report = await RuleAdmin(_Engine(_report())).importRules('x');
    expect(report.imported, 1);
    expect(report.alreadyPresent, 1);
    expect(report.refused, 2);
    expect(report.replaced, 0);
    final refused = report.entries.where((e) => e.refused).toList();
    expect(refused[0].reason, 'unknown_label');
    expect(refused[1].reason, 'invalid_rule');
    expect(refused[1].code, 'invalid_input.parameter');
    expect(report.entries[1].position, 2);
    expect(report.entries[1].name, 'Cassée');
  });
}
