import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/template_admin.dart';
import 'package:mira/state/templates.dart';

import 'fake_template_engine.dart';

void main() {
  late FakeTemplateEngine engine;

  setUp(() => engine = FakeTemplateEngine());

  test('reload charge la liste du moteur', () async {
    engine.templates.add({
      'id': 't1',
      'name': 'Merci',
      'subject': null,
      'body': 'Corps',
      'created_at': 1,
      'updated_at': 1,
    });
    final state = Templates(TemplateAdmin(engine), const Stream.empty());
    expect(state.items, isEmpty);
    await state.reload();
    expect(state.items.map((t) => t.name), ['Merci']);
    expect(state.error, isNull);
    state.dispose();
  });

  test('un événement templates_changed relit la liste', () async {
    final changes = StreamController<void>.broadcast();
    addTearDown(changes.close);
    final state = Templates(TemplateAdmin(engine), changes.stream);
    await state.reload();
    final before = engine.requestsOf('list_templates').length;
    changes.add(null);
    await Future<void>.delayed(Duration.zero);
    await Future<void>.delayed(Duration.zero);
    expect(engine.requestsOf('list_templates'), hasLength(before + 1));
    state.dispose();
  });

  test('une erreur du moteur est exposée sans vider la liste', () async {
    engine.templates.add({
      'id': 't1',
      'name': 'Merci',
      'subject': null,
      'body': 'Corps',
      'created_at': 1,
      'updated_at': 1,
    });
    final state = Templates(TemplateAdmin(engine), const Stream.empty());
    await state.reload();
    engine.failNext = const EngineException('internal.bug', 'x');
    await state.reload();
    expect(state.items.map((t) => t.name), ['Merci']);
    expect(state.error, isA<EngineException>());
    await state.reload();
    expect(state.error, isNull);
    state.dispose();
  });
}
