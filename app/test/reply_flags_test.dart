import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/state/identity.dart';
import 'package:mira/state/automated_senders.dart';
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

    test('avant le délai : pas encore en attente ; au délai : en attente', () {
      final sent = mail('sent', 't1', [msg(noah, 1000, 'Question')], to: [ana]);
      final day = const Duration(days: 1).inMilliseconds;
      markMentionsAndWaiting([sent], me, now: 1000 + day);
      expect(sent.flags, isNot(contains('waiting')));
      markMentionsAndWaiting([sent], me, now: 1000 + 2 * day);
      expect(sent.flags, contains('waiting'));
    });

    test('destinataire automatique (noreply) : jamais en attente', () {
      const bot = Person('Bot', 'no-reply@service.com');
      final sent = mail('sent', 't1', [msg(noah, 1, 'Demande')], to: [bot]);
      markMentionsAndWaiting([sent], me);
      expect(sent.flags, isNot(contains('waiting')));
    });

    test('un humain parmi les destinataires suffit', () {
      const bot = Person('Bot', 'noreply@service.com');
      final sent = mail(
        'sent',
        't1',
        [msg(noah, 1, 'Demande')],
        to: [bot, ana],
      );
      markMentionsAndWaiting([sent], me);
      expect(sent.flags, contains('waiting'));
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

    test('expéditeur automatique qui cite mon adresse : pas une mention', () {
      const bot = Person('Tally', 'notifications@tally.so');
      final m = mail('inbox', 't1', [msg(bot, 1, 'Envoyé à noah@mira.dev')]);
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

  test('isAutomated reconnaît les adresses d\'envoi automatique', () {
    for (final a in [
      'noreply@x.com',
      'No-Reply@x.com',
      'do_not_reply@x.com',
      'notifications@tally.so',
      'notification+abc@x.com',
      'mailer-daemon@x.com',
      'bounce-123@x.com',
    ]) {
      expect(isAutomated(a), isTrue, reason: a);
    }
    for (final a in ['ana@ex.org', 'reply@x.com', 'noah.reply@x.com', 'bob']) {
      expect(isAutomated(a), isFalse, reason: a);
    }
  });
}
