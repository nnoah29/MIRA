import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/state/alias_hints.dart';

Mail _mail(String folder, Person from, List<Person> to) => Mail(
  id: '$folder-${from.email}-${to.map((p) => p.email).join()}',
  folder: folder,
  from: from,
  to: to,
  subject: 's',
  date: 0,
  msgs: [Msg(from: from, date: 0, body: const [])],
  account: 'a1',
);

void main() {
  const me = Person('Moi', 'moi@gmail.com');
  const alias = Person('Moi', 'iam@perso.com');
  const bob = Person('Bob', 'bob@x.org');

  test(
    'un expéditeur de messages envoyés et un destinataire unique sont proposés',
    () {
      final hints = aliasHints(
        [
          _mail('sent', alias, [bob]),
          _mail('sent', alias, [bob]),
          _mail('inbox', bob, [const Person('Hello', 'hello@perso.com')]),
        ],
        {'moi@gmail.com'},
      );
      expect(hints.map((h) => h.address), ['iam@perso.com', 'hello@perso.com']);
      expect(hints.first.sentAs, 2);
      expect(hints.last.receivedAt, 1);
    },
  );

  test('adresses connues, courriers collectifs et brouillons sont ignorés', () {
    final hints = aliasHints(
      [
        _mail('sent', me, [bob]),
        _mail('inbox', bob, [me]),
        _mail('inbox', bob, [alias, me]),
        _mail('inbox', bob, [alias, bob]),
        _mail('drafts', alias, [bob]),
        _mail('spam', bob, [const Person('X', 'spam@perso.com')]),
      ],
      {'moi@gmail.com', 'iam@perso.com'},
    );
    expect(hints, isEmpty);
  });
}
