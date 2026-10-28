import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/state/reply_flags.dart';

const me = Identity(
  {'noah@mira.dev', 'alias@mira.dev'},
  {'noah toffa', 'noah'},
);
const ana = Person('Ana', 'ana@ex.org');
const noah = Person('Noah Toffa', 'noah@mira.dev');

Msg msg(Person from, int date, String text) => Msg(
  from: from,
  date: date,
  body: [
    Para([Span(text)]),
  ],
);

Mail mail(
  String folder,
  String thread,
  List<Msg> msgs, {
  List<Person> to = const [noah],
  List<Person> cc = const [],
  List<String> flags = const [],
}) => Mail(
  id: 'a1/$folder/$thread',
  folder: folder,
  from: msgs.last.from,
  to: to,
  cc: cc,
  subject: 'sujet',
  date: msgs.last.date,
  msgs: msgs,
  flags: [...flags],
);

void main() {
  group('waiting', () {
    test(
      'mon message est le dernier du fil : en attente, dans les deux dossiers',
      () {
        final inbox = mail('inbox', 't1', [msg(ana, 1, 'Salut')]);
        final sent = mail('sent', 't1', [msg(noah, 2, 'Réponse')], to: [ana]);
        markMentionsAndWaiting([inbox, sent], me);
        expect(inbox.flags, contains('waiting'));
        expect(sent.flags, contains('waiting'));
      },
    );

    test('un message reçu après le mien lève l\'attente', () {
      final sent = mail('sent', 't1', [msg(noah, 1, 'Question')], to: [ana]);
      final inbox = mail('inbox', 't1', [msg(ana, 2, 'Voilà')]);
      markMentionsAndWaiting([inbox, sent], me);
      expect(inbox.flags, isNot(contains('waiting')));
      expect(sent.flags, isNot(contains('waiting')));
    });

    test('un message envoyé seulement à soi-même n\'attend rien', () {
      final sent = mail('sent', 't1', [msg(noah, 1, 'Note')], to: [noah]);
      markMentionsAndWaiting([sent], me);
      expect(sent.flags, isNot(contains('waiting')));
    });

    test('brouillons, corbeille et spam sont ignorés', () {
      final draft = mail(
        'drafts',
        't1',
        [msg(noah, 1, 'Brouillon')],
        to: [ana],
      );
      final trash = mail('trash', 't2', [msg(noah, 1, 'Jeté')], to: [ana]);
      markMentionsAndWaiting([draft, trash], me);
      expect(draft.flags + trash.flags, isEmpty);
    });

    test('un marqueur périmé est retiré, les autres sont conservés', () {
      final inbox = mail(
        'inbox',
        't1',
        [msg(ana, 1, 'Salut')],
        flags: ['waiting', 'needs'],
      );
      markMentionsAndWaiting([inbox], me);
      expect(inbox.flags, ['needs']);
    });
  });

  group('mention', () {
    test('@prénom dans le dernier message reçu', () {
      final m = mail('inbox', 't1', [msg(ana, 1, 'Merci @Noah de regarder')]);
      markMentionsAndWaiting([m], me);
      expect(m.flags, contains('mention'));
    });

    test('mon adresse citée dans le texte', () {
      final m = mail('inbox', 't1', [msg(ana, 1, 'Écris à alias@mira.dev')]);
      markMentionsAndWaiting([m], me);
      expect(m.flags, contains('mention'));
    });

    test('simple salutation ou Cc : pas une mention', () {
      final m = mail(
        'inbox',
        't1',
        [msg(ana, 1, 'Bonjour Noah, ça va ?')],
        cc: [noah],
      );
      markMentionsAndWaiting([m], me);
      expect(m.flags, isNot(contains('mention')));
    });

    test('adresse d\'un autre domaine ressemblant : pas une mention', () {
      final m = mail('inbox', 't1', [msg(ana, 1, 'Voir joe@noahcorp.com')]);
      markMentionsAndWaiting([m], me);
      expect(m.flags, isNot(contains('mention')));
    });

    test('texte cité (lignes >) ignoré', () {
      final m = mail('inbox', 't1', [msg(ana, 1, '> @noah regarde\nOK')]);
      markMentionsAndWaiting([m], me);
      expect(m.flags, isNot(contains('mention')));
    });

    test('ma propre réponse ensuite lève la mention', () {
      final inbox = mail('inbox', 't1', [msg(ana, 1, '@noah un avis ?')]);
      final sent = mail('sent', 't1', [msg(noah, 2, 'Oui')], to: [ana]);
      markMentionsAndWaiting([inbox, sent], me);
      expect(inbox.flags, isNot(contains('mention')));
    });
  });
}
