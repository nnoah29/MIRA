import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/template_admin.dart';

import 'fake_template_engine.dart';

void main() {
  late FakeTemplateEngine engine;
  late TemplateAdmin admin;

  setUp(() {
    engine = FakeTemplateEngine();
    admin = TemplateAdmin(engine);
  });

  test('list décode les modèles du moteur', () async {
    engine.templates.add({
      'id': 't1',
      'name': 'Merci',
      'subject': null,
      'body': 'Bonjour {{contact.nom}}',
      'created_at': 1,
      'updated_at': 2,
    });
    final found = await admin.list();
    expect(engine.calls.single['domain'], 'template');
    expect(engine.requestsOf('list_templates'), hasLength(1));
    final one = found.single;
    expect((one.id, one.name, one.subject), ('t1', 'Merci', null));
    expect(one.body, 'Bonjour {{contact.nom}}');
  });

  test(
    'create envoie une clé d’idempotence différente à chaque appel',
    () async {
      await admin.create('Merci', 'Objet', 'Corps');
      await admin.create('Relance', null, 'Corps');
      final sent = engine.requestsOf('create_template');
      expect(sent[0]['name'], 'Merci');
      expect(sent[0]['subject'], 'Objet');
      expect(sent[1]['subject'], isNull);
      expect(sent[0]['idempotency_key'], isNotEmpty);
      expect(sent[0]['idempotency_key'], isNot(sent[1]['idempotency_key']));
    },
  );

  test('update et delete visent le modèle par son identifiant', () async {
    final made = await admin.create('Merci', null, 'Corps');
    await admin.update(made.id, 'Merci 2', 'Objet', 'Nouveau');
    await admin.delete(made.id);
    final update = engine.requestsOf('update_template').single;
    expect(update['template'], made.id);
    expect(
      (update['name'], update['subject'], update['body']),
      ('Merci 2', 'Objet', 'Nouveau'),
    );
    expect(engine.requestsOf('delete_template').single['template'], made.id);
  });

  test('render envoie le contexte et décode les variables restantes', () async {
    final made = await admin.create('Merci', 'Objet', 'Corps');
    engine.templates.single['body'] = 'Bonjour {{contact.nom}}';
    final rendered = await admin.render(made.id, {'date': '25/10/2026'});
    final sent = engine.requestsOf('render_template').single;
    expect(sent['context'], {'date': '25/10/2026'});
    expect(rendered.body, 'Bonjour {{contact.nom}}');
    expect(rendered.subject, 'Objet');
    expect(rendered.unresolved, ['contact.nom']);
  });
}
