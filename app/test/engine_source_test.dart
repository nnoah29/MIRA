import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/engine_mail_source.dart';
import 'package:mira/state/store.dart';
import 'package:shared_preferences/shared_preferences.dart';

class FakeEngine implements EngineClient {
  final calls = <String>[];
  final bodyFails = <String>{};
  bool allowed = false;
  bool extra = false;
  final feed = StreamController<Map<String, dynamic>>.broadcast();

  Map<String, dynamic> _msg(
    String id,
    String subject, {
    String? thread,
    bool seen = true,
    int date = 1000,
    bool attachments = false,
  }) => {
    'id': id,
    'account': 'acc1',
    'thread': thread,
    'from': [
      {'name': 'Bob', 'address': 'bob@example.org'},
    ],
    'to': [
      {'name': null, 'address': 'alice@example.org'},
    ],
    'cc': [],
    'subject': subject,
    'date': date,
    'internal_date': date,
    'flags': {
      'seen': seen,
      'flagged': false,
      'answered': false,
      'forwarded': false,
    },
    'preview': 'Aperçu $subject',
    'has_attachments': attachments,
    'content': 'headers_only',
    'mailboxes': [],
  };

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async {
    final r = request['request'] as Map<String, dynamic>;
    final op = '${request['domain']}.${r['op']}';
    calls.add(op);
    switch (op) {
      case 'account.list_accounts':
        return {
          'type': 'accounts',
          'data': [
            {
              'id': 'acc1',
              'display_name': 'Alice',
              'address': 'alice@example.org',
            },
          ],
        };
      case 'mailbox.list_mailboxes':
        return {
          'type': 'mailboxes',
          'data': [
            {'id': 'mb-inbox', 'role': 'inbox'},
            {'id': 'mb-junk', 'role': 'junk'},
            {'id': 'mb-other', 'role': 'other'},
          ],
        };
      case 'message.list_messages':
        return _list('${(r['view'] as Map)['mailbox']}');
      case 'message.get_message_body':
        if (bodyFails.contains(r['message'])) {
          throw const EngineException(
            'incompatible_state.body_unavailable_offline',
            'absent',
          );
        }
        return {
          'type': 'message_body',
          'data': {
            'text': 'Un\n\nDeux',
            'html_available': r['message'] == 'm2',
          },
        };
      case 'message.fetch_message_body':
        return {
          'type': 'message_body',
          'data': {'text': 'Téléchargé', 'html_available': false},
        };
      case 'message.get_safe_html':
        const table = '<table><tr><td>Bonjour <b>Alice</b></td></tr></table>';
        return {
          'type': 'safe_html',
          'data': {
            'html': allowed
                ? '$table<img src="cid:logo@x"><img src="https://x.test/a.png">'
                : '$table<img src="cid:logo@x">',
            'blocked_resources': allowed ? [] : ['https://x.test/a.png'],
          },
        };
      case 'identity.list_identities':
        return {
          'type': 'identities',
          'data': [
            {
              'id': 'i1',
              'account': 'acc1',
              'address': 'alice@example.org',
              'display_name': 'Alice',
              'signature': null,
              'is_default': false,
            },
            {
              'id': 'i2',
              'account': 'acc1',
              'address': 'iam@perso.com',
              'display_name': 'Alice Pro',
              'signature': 'Alice, Pro',
              'is_default': true,
            },
          ],
        };
      case 'sync.start_sync':
        return {'type': 'done'};
      case 'message.allow_remote_content':
        allowed = true;
        return {'type': 'done'};
      case 'message.save_attachment':
        File(r['destination'] as String).writeAsBytesSync([1, 2, 3]);
        return {
          'type': 'operation',
          'data': {'id': 'op1'},
        };
      case 'message.list_attachments':
        return {
          'type': 'attachments',
          'data': [
            {
              'id': 'a1',
              'filename': 'devis.pdf',
              'mime_type': 'application/pdf',
              'size': 2048,
            },
            {
              'id': 'a2',
              'filename': 'logo.png',
              'mime_type': 'image/png',
              'size': 3,
              'content_id': '<logo@x>',
              'inline': true,
            },
          ],
        };
    }
    throw EngineException('internal.bug', 'op inconnue $op');
  }

  Map<String, dynamic> _list(String mailbox) {
    final items = switch (mailbox) {
      'mb-inbox' => [
        if (extra) _msg('m9', 'Nouveau', date: 4000),
        _msg('m1', 'Premier', thread: 't1', seen: false, date: 1000),
        _msg('m2', 'Re: Premier', thread: 't1', date: 2000, attachments: true),
        _msg('m3', 'Seul', date: 3000),
      ],
      'mb-junk' => [_msg('m4', 'Pourriel', date: 500)],
      _ => <Map<String, dynamic>>[],
    };
    return {
      'type': 'messages',
      'data': {'items': items, 'next_cursor': null},
    };
  }

  @override
  Stream<Map<String, dynamic>> events({int? after}) => feed.stream;

  @override
  Future<String?> originalHtml(String messageId) async =>
      '<html><head><style>.t{color:#ff0000}</style></head>'
      '<body><p class="t">Rouge</p><img src="cid:logo@x"></body></html>';

  @override
  Future<void> close() async {}
}

void main() {
  test('les comptes et boîtes viennent du moteur, par dossier', () async {
    final src = await EngineMailSource.load(FakeEngine());
    expect(src.accounts.single.email, 'alice@example.org');
    final box = src.boxes['acc1']!;
    expect(box.where((m) => m.folder == 'inbox').length, 2);
    expect(box.where((m) => m.folder == 'spam').length, 1);
    expect(box.any((m) => m.folder == 'archive'), isFalse);
  });

  test('deux messages du même fil forment une conversation', () async {
    final src = await EngineMailSource.load(FakeEngine());
    final conv = src.boxes['acc1']!.firstWhere((m) => m.msgs.length == 2);
    expect(conv.subject, 'Re: Premier');
    expect(conv.unread, isTrue);
    expect(conv.msgs.first.ref, 'm1');
    expect(conv.from.email, 'bob@example.org');
  });

  test('hydrate charge le HTML sûr, le texte et les pièces jointes', () async {
    final src = await EngineMailSource.load(FakeEngine());
    final conv = src.boxes['acc1']!.firstWhere((m) => m.msgs.length == 2);
    expect(await src.hydrate(conv), isTrue);
    expect(conv.msgs[0].body.length, 2);
    expect(conv.msgs[1].html, contains('color:#ff0000'));
    expect(conv.msgs[1].html, contains('Rouge'));
    expect(conv.msgs[1].blocked, ['https://x.test/a.png']);
    expect(conv.msgs[1].inline['logo@x'], [1, 2, 3]);
    expect(conv.msgs[1].att.single.n, 'devis.pdf');
    expect(conv.msgs[1].att.single.t, 'PDF');
    expect(await src.hydrate(conv), isFalse);
  });

  test('autoriser le contenu distant recharge un HTML complet', () async {
    final fake = FakeEngine();
    final src = await EngineMailSource.load(fake);
    final conv = src.boxes['acc1']!.firstWhere((m) => m.msgs.length == 2);
    await src.hydrate(conv);
    await src.allowRemote(conv.msgs[1]);
    expect(conv.msgs[1].blocked, isEmpty);
    expect(fake.calls, contains('message.allow_remote_content'));
  });

  test('corps absent hors ligne : demande explicite au serveur', () async {
    final fake = FakeEngine()..bodyFails.add('m3');
    final src = await EngineMailSource.load(fake);
    final solo = src.boxes['acc1']!.firstWhere((m) => m.subject == 'Seul');
    await src.hydrate(solo);
    expect(fake.calls, contains('message.fetch_message_body'));
    expect(solo.msgs.single.body.single, isNotNull);
  });

  test('le store affiche les données du moteur, sans mails fictifs', () async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();
    final src = await EngineMailSource.load(FakeEngine());
    final store = MailStore(prefs, source: src);
    expect(store.accounts.single.id, 'acc1');
    expect(store.mails.length, 3);
    expect(store.visible().map((m) => m.subject), contains('Seul'));
    store.send();
    store.dispose();
  });

  test('un rechargement de la boîte garde le HTML déjà chargé', () async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();
    final fake = FakeEngine();
    final src = await EngineMailSource.load(
      fake,
      debounce: const Duration(milliseconds: 20),
    );
    final store = MailStore(prefs, source: src);
    final conv = store.mails.firstWhere((m) => m.msgs.length == 2);
    store.openMail(conv.id);
    await Future<void>.delayed(const Duration(milliseconds: 300));
    expect(conv.msgs[1].html, isNotNull);
    fake.extra = true;
    fake.feed.add({
      'kind': {'type': 'sync_finished', 'account': 'acc1'},
    });
    await Future<void>.delayed(const Duration(milliseconds: 300));
    final fresh = store.mails.firstWhere((m) => m.msgs.length == 2);
    expect(identical(fresh, conv), isFalse);
    expect(fresh.msgs[1].html, contains('Rouge'));
    expect(fresh.msgs[1].inline, isNotEmpty);
    expect(fresh.unread, isFalse);
    store.dispose();
  });

  test('une rafale d\u2019événements ne provoque qu\u2019une relecture, sans redessin si rien ne change', () async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();
    final fake = FakeEngine();
    final src = await EngineMailSource.load(
      fake,
      debounce: const Duration(milliseconds: 50),
    );
    final store = MailStore(prefs, source: src);
    var redraws = 0;
    store.addListener(() => redraws++);
    final before = fake.calls
        .where((c) => c == 'mailbox.list_mailboxes')
        .length;
    for (var i = 0; i < 200; i++) {
      fake.feed.add({
        'kind': {'type': 'sync_finished', 'account': 'acc1'},
      });
      fake.feed.add({
        'kind': {'type': 'body_available', 'account': 'acc1'},
      });
    }
    await Future<void>.delayed(const Duration(milliseconds: 400));
    final reads = fake.calls.where((c) => c == 'mailbox.list_mailboxes').length;
    expect(reads - before, 1);
    expect(redraws, 0);
    store.dispose();
  });

  test(
    'ouvrir un message charge aussi ses voisins (j/k instantanés)',
    () async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final fake = FakeEngine();
      final src = await EngineMailSource.load(fake);
      final store = MailStore(prefs, source: src);
      final l = store.visible();
      expect(l.length, greaterThan(1));
      store.openMail(l.first.id);
      await Future<void>.delayed(const Duration(milliseconds: 200));
      expect(l.first.msgs.every((m) => m.loaded), isTrue);
      expect(l[1].msgs.every((m) => m.loaded), isTrue);
      store.dispose();
    },
  );

  test(
    'le corps est annoncé avant les pièces jointes et images incorporées',
    () async {
      final src = await EngineMailSource.load(FakeEngine());
      final conv = src.boxes['acc1']!.firstWhere((m) => m.msgs.length == 2);
      final seen = <String>[];
      await src.hydrate(
        conv,
        onProgress: () => seen.add(
          'loaded=${conv.msgs[1].loaded} att=${conv.msgs[1].att.length}',
        ),
      );
      expect(seen, contains('loaded=true att=0'));
      expect(conv.msgs[1].att, isNotEmpty);
    },
  );

  test(
    'Actualiser demande une synchro de tous les comptes, sans rafale',
    () async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final fake = FakeEngine();
      final src = await EngineMailSource.load(fake);
      final store = MailStore(prefs, source: src);
      await store.refreshNow();
      final sync = fake.calls.where((c) => c == 'sync.start_sync').length;
      expect(sync, 1);
      expect(store.syncing, isFalse);
      await store.refreshNow(quiet: true);
      expect(
        fake.calls.where((c) => c == 'sync.start_sync').length,
        1,
        reason: 'le retour de focus ne relance pas avant 30 s',
      );
      store.dispose();
    },
  );

  test('les identités alimentent le composeur : défaut, adresse de réception, signature', () async {
    SharedPreferences.setMockInitialValues({'sig': 'Signature générale'});
    final prefs = await SharedPreferences.getInstance();
    final src = await EngineMailSource.load(FakeEngine());
    final store = MailStore(prefs, source: src);
    final choices = store.senderChoices().map((s) => s.email).toList();
    expect(choices.first, 'iam@perso.com');
    expect(choices, contains('alice@example.org'));
    expect(store.fromFor(), 'iam@perso.com');
    final viaMain = store.mails.first
      ..to = [const Person('Alice', 'alice@example.org')];
    expect(store.fromFor(viaMain), 'alice@example.org');
    final viaAlias = store.mails.last
      ..to = [const Person('Alice Pro', 'iam@perso.com')];
    expect(store.fromFor(viaAlias), 'iam@perso.com');
    expect(store.signatureFor('iam@perso.com'), 'Alice, Pro');
    expect(store.signatureFor('alice@example.org'), 'Signature générale');
    expect(isMine('IAM@perso.com'), isTrue);
    store.openCompose(const ComposeInit());
    expect(store.c!.from, 'iam@perso.com');
    store.dispose();
    knownSenders = [];
  });
}
