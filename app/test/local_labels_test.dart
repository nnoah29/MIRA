import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/state/local_labels.dart';

Mail _mail(String id) => Mail(
  id: id,
  folder: 'inbox',
  from: const Person('A', 'a@x.org'),
  to: const [],
  subject: 's',
  date: 0,
  msgs: [],
);

void main() {
  test('création : nom vide ou déjà pris refusé, clés locales distinctes', () {
    final l = LocalLabels();
    final a = l.create('Perso');
    expect(a, startsWith(localLabelPrefix));
    expect(l.create('  '), isNull);
    expect(l.create('perso'), isNull);
    expect(l.create('Travail'), isNot(a));
    expect(l.defs[a]!.local, isTrue);
  });

  test('la pose se reporte sur les conversations et survit à l\'encodage', () {
    final l = LocalLabels();
    final k = l.create('Perso')!;
    l.set(['m1', 'm2'], k, true);
    final back = LocalLabels.decode(l.encode());
    final mails = [_mail('m1'), _mail('m2'), _mail('m3')];
    back.apply(mails);
    expect(mails.map((m) => m.labels), [
      [k],
      [k],
      <String>[],
    ]);
    expect(back.count(k), 2);
  });

  test('supprimer un libellé le retire des conversations', () {
    final l = LocalLabels();
    final k = l.create('Perso')!;
    l.set(['m1'], k, true);
    l.delete(k);
    final m = _mail('m1')..labels = ['INBOX', k];
    l.apply([m]);
    expect(m.labels, ['INBOX']);
  });

  test('les libellés du serveur sont conservés par apply', () {
    final l = LocalLabels();
    final k = l.create('Perso')!;
    l.set(['m1'], k, true);
    final m = _mail('m1')..labels = ['srv-1'];
    l.apply([m]);
    expect(m.labels, ['srv-1', k]);
  });

  test('un état illisible repart de zéro', () {
    expect(LocalLabels.decode('{nope').defs, isEmpty);
  });
}
