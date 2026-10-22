import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/local_label_admin.dart';

/// Moteur factice : garde les requêtes et rend des réponses du domaine `local_label`.
class FakeLabelEngine implements EngineClient {
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
  late FakeLabelEngine engine;
  late LocalLabelAdmin admin;

  setUp(() {
    engine = FakeLabelEngine();
    admin = LocalLabelAdmin(engine);
  });

  test('list lit les libellés du moteur', () async {
    engine.answer = {
      'type': 'labels',
      'data': [
        {'id': 'l1', 'name': 'À payer', 'color': 4293874731},
      ],
    };
    final labels = await admin.list();
    expect(engine.sent.single['domain'], 'local_label');
    expect(engine.last['op'], 'list_local_labels');
    expect(
      (labels.single.id, labels.single.name, labels.single.color),
      ('l1', 'À payer', 4293874731),
    );
  });

  test('assignments rend les libellés de chaque conversation', () async {
    engine.answer = {
      'type': 'assignments',
      'data': [
        {
          'thread': 't1',
          'labels': ['l1', 'l2'],
        },
      ],
    };
    final poses = await admin.assignments();
    expect(engine.last['op'], 'list_local_label_assignments');
    expect(poses, {
      't1': {'l1', 'l2'},
    });
  });

  test('create envoie nom, couleur et une clé d’idempotence neuve', () async {
    engine.answer = {
      'type': 'label',
      'data': {'id': 'l9', 'name': 'Urgent', 'color': 1},
    };
    final first = await admin.create('Urgent', 1);
    final firstKey = engine.last['idempotency_key'];
    await admin.create('Urgent', 1);
    expect(first.id, 'l9');
    expect(engine.last['op'], 'create_local_label');
    expect((engine.last['name'], engine.last['color']), ('Urgent', 1));
    expect(firstKey, isNotEmpty);
    expect(engine.last['idempotency_key'], isNot(firstKey));
  });

  test(
    'rename, recolor et delete visent le libellé par son identifiant',
    () async {
      engine.answer = {
        'type': 'label',
        'data': {'id': 'l1', 'name': 'X', 'color': 2},
      };
      await admin.rename('l1', 'X');
      expect(engine.last, {
        'op': 'rename_local_label',
        'label': 'l1',
        'name': 'X',
      });
      await admin.recolor('l1', 2);
      expect(engine.last, {
        'op': 'recolor_local_label',
        'label': 'l1',
        'color': 2,
      });
      engine.answer = {'type': 'done'};
      await admin.delete('l1');
      expect(engine.last, {'op': 'delete_local_label', 'label': 'l1'});
    },
  );

  test('set pose ou retire sur des conversations', () async {
    await admin.set('l1', ['t1', 't2'], true);
    expect(engine.last['op'], 'set_local_label');
    expect(engine.last['label'], 'l1');
    expect(engine.last['threads'], ['t1', 't2']);
    expect(engine.last['on'], true);
    expect(engine.last['idempotency_key'], isNotEmpty);
  });

  test(
    'importLabels envoie libellés et conversations, rend le bilan',
    () async {
      engine.answer = {
        'type': 'imported',
        'data': {'labels': 1, 'assignments': 2, 'ignored': 1},
      };
      final report = await admin.importLabels([
        const LocalLabelImport('À payer', 4293874731, ['t1', 't2', 'inconnue']),
      ]);
      expect(engine.last['op'], 'import_local_labels');
      final sent = (engine.last['labels'] as List).single as Map;
      expect(sent, {
        'name': 'À payer',
        'color': 4293874731,
        'conversations': ['t1', 't2', 'inconnue'],
      });
      expect((report.labels, report.assignments, report.ignored), (1, 2, 1));
    },
  );
}
