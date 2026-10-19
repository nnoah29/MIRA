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

  test('mailto seul : un message prérempli s\'ouvre', () async {
    final s = await start();
    await s.unsubscribe(_mail({'mailto': 'leave@x.org?subject=stop'}));
    expect(s.c!.to.single.email, 'leave@x.org');
    expect(s.c!.subject.text, 'stop');
    s.dispose();
  });
}
