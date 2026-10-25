import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/state/template_context.dart';

void main() {
  final now = DateTime(2026, 10, 5, 9, 7);

  Map<String, dynamic> build({
    List<Person> to = const [],
    String subject = '',
    String? fromName,
  }) => buildTemplateContext(
    to: to,
    subject: subject,
    fromEmail: 'noah@x.fr',
    fromName: fromName,
    now: now,
  );

  test('le premier destinataire fournit nom, prénom et adresse', () {
    final context = build(
      to: const [
        Person('Léa Durand', 'lea@x.fr'),
        Person('Autre', 'autre@x.fr'),
      ],
    );
    expect(context['contact'], {
      'name': 'Léa Durand',
      'first_name': 'Léa',
      'email': 'lea@x.fr',
    });
  });

  test('sans nom affiché, seule l’adresse est transmise', () {
    expect(build(to: const [Person('', 'lea@x.fr')])['contact'], {
      'email': 'lea@x.fr',
    });
    expect(build(to: const [Person('lea@x.fr', 'lea@x.fr')])['contact'], {
      'email': 'lea@x.fr',
    });
  });

  test('sans destinataire, aucune clé contact', () {
    expect(build().containsKey('contact'), isFalse);
  });

  test('date et heure sont formatées par l’app, les valeurs vides omises', () {
    final context = build(subject: '  ', fromName: '');
    expect(context['date'], '05/10/2026');
    expect(context['time'], '09:07');
    expect(context['account_email'], 'noah@x.fr');
    expect(context.containsKey('subject'), isFalse);
    expect(context.containsKey('account_name'), isFalse);
  });

  test('objet et nom de l’identité sont transmis quand ils existent', () {
    final context = build(subject: 'Devis', fromName: 'Noah Toffa');
    expect(context['subject'], 'Devis');
    expect(context['account_name'], 'Noah Toffa');
  });
}
