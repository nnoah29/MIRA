import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/engine/mappers.dart';
import 'package:mira/state/store.dart';
import 'package:shared_preferences/shared_preferences.dart';

Mail _mail(Map<String, dynamic>? unsub) => Mail(
  id: 'a/inbox/t',
  folder: 'inbox',
  from: const Person('News', 'news@x.org'),
  to: const [Person('Moi', 'me@x.org')],
  subject: 'Lettre',
  date: 1,
  unread: true,
  msgs: [
    msgOf({
      'id': 'm1',
      'from': [
        {'name': 'News', 'address': 'news@x.org'},
      ],
      'date': 1,
      'unsubscribe': unsub,
    }),
  ],
  account: 'a',
);

void main() {
  Future<MailStore> start() async {
    SharedPreferences.setMockInitialValues({});
    return MailStore(await SharedPreferences.getInstance());
  }

  test('sans en-tête de désabonnement, rien n\'est proposé', () async {
    final s = await start();
    expect(s.unsubFor(_mail(null)), isNull);
    s.dispose();
  });

  test('un clic : appel direct, sans ouvrir de page', () async {
    final s = await start();
    final posted = <Uri>[];
    s.unsubscriber.postOneClick = (u) async {
      posted.add(u);
      return true;
    };
    s.unsubscriber.openPage = (u) async => fail('page ouverte');
    await s.unsubscribe(
      _mail({'https_url': 'https://x.org/u?id=1', 'one_click': true}),
    );
    expect(posted.single.toString(), 'https://x.org/u?id=1');
    expect(s.toasts.last.msg, 'Désabonnement envoyé');
    s.dispose();
  });

  test('sans un clic : la page s\'ouvre dans le navigateur', () async {
    final s = await start();
    final opened = <Uri>[];
    s.unsubscriber.openPage = (u) async {
      opened.add(u);
      return true;
    };
    await s.unsubscribe(
      _mail({'https_url': 'https://x.org/u', 'one_click': false}),
    );
    expect(opened.single.host, 'x.org');
    s.dispose();
  });

  test('un clic accepté : l\'expéditeur est retenu comme désabonné', () async {
    final s = await start();
    s.unsubscriber.postOneClick = (u) async => true;
    final m = _mail({'https_url': 'https://x.org/u', 'one_click': true});
    await s.unsubscribe(m);
    expect(s.isUnsubscribed(m), isTrue);
    expect(s.prefs.getStringList('unsubscribed'), ['news@x.org']);
    s.dispose();
  });

  test('un clic refusé : rien n\'est retenu, l\'échec est signalé', () async {
    final s = await start();
    s.unsubscriber.postOneClick = (u) async => false;
    final m = _mail({'https_url': 'https://x.org/u', 'one_click': true});
    await s.unsubscribe(m);
    expect(s.isUnsubscribed(m), isFalse);
    expect(s.toasts.last.msg, 'Désabonnement impossible');
    s.dispose();
  });

  test(
    'page seule : rien n\'est retenu (le désabonnement se finit sur le site)',
    () async {
      final s = await start();
      s.unsubscriber.openPage = (u) async => true;
      final m = _mail({'https_url': 'https://x.org/u', 'one_click': false});
      await s.unsubscribe(m);
      expect(s.isUnsubscribed(m), isFalse);
      s.dispose();
    },
  );

  test('mailto : le message part sans ouvrir la rédaction', () async {
    final s = await start();
    final m = _mail({'mailto': 'leave@x.org?subject=stop'});
    await s.unsubscribe(m);
    expect(s.c, isNull);
    expect(s.isUnsubscribed(m), isTrue);
    expect(s.toasts.last.msg, 'Désabonnement envoyé');
    s.dispose();
  });

  test(
    'mailto préféré à une page sans un clic : tout se fait dans l\'app',
    () async {
      final s = await start();
      s.unsubscriber.openPage = (u) async => fail('page ouverte');
      final m = _mail({
        'https_url': 'https://x.org/u',
        'one_click': false,
        'mailto': 'leave@x.org',
      });
      await s.unsubscribe(m);
      expect(s.isUnsubscribed(m), isTrue);
      s.dispose();
    },
  );

  test('désabonnement dans l\'app : un clic ou une adresse exploitable', () {
    expect(const Unsub(url: 'https://x.org', oneClick: true).inApp, isTrue);
    expect(const Unsub(mailto: 'leave@x.org').inApp, isTrue);
    expect(const Unsub(url: 'https://x.org').inApp, isFalse);
    expect(const Unsub(mailto: 'mailto:').inApp, isFalse);
  });
}
