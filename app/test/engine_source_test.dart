import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/engine_mail_source.dart';
import 'package:mira/state/folder_map.dart';
import 'package:mira/state/store.dart';
import 'package:shared_preferences/shared_preferences.dart';

class FakeEngine implements EngineClient {
  final calls = <String>[];
  final bodyFails = <String>{};
  bool allowed = false;
  bool extra = false;
  bool second = false;
  bool labelOnly = false;
  bool serverCopy = false;
  final actions = <Map<String, dynamic>>[];
  final composed = <Map<String, dynamic>>[];
  final drafts = <String, Map<String, dynamic>>{};
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
            if (second)
              {'id': 'acc2', 'display_name': 'Pro', 'address': 'alice@pro.org'},
          ],
        };
      case 'mailbox.list_mailboxes':
        if (r['account'] == 'acc2') {
          return {
            'type': 'mailboxes',
            'data': [
              {'id': 'mb2-inbox', 'role': 'inbox'},
              {
                'id': 'mb2-other',
                'role': 'other',
                'name': 'Factures',
                'display_name': 'Factures',
              },
            ],
          };
        }
        return {
          'type': 'mailboxes',
          'data': [
            {'id': 'mb-inbox', 'role': 'inbox'},
            {'id': 'mb-junk', 'role': 'junk'},
            {'id': 'mb-drafts', 'role': 'drafts'},
            {
              'id': 'mb-other',
              'role': 'other',
              'name': 'Factures',
              'display_name': 'Factures',
            },
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
      case 'action.apply_action' ||
          'action.create_mailbox' ||
          'action.rename_mailbox' ||
          'action.delete_mailbox':
        actions.add({'op': r['op'], ...r});
        return {
          'type': 'write',
          'data': {'type': 'action', 'data': 'a1'},
        };
      case 'compose.create_draft':
        composed.add({'op': r['op'], ...r});
        drafts['dr1'] = {
          'id': 'dr1',
          'account': r['account'],
          'identity': r['identity'] ?? 'id1',
          'to': [],
          'cc': [],
          'bcc': [],
          'subject': '',
          'body_text': '',
          'body_html': null,
          'attachments': [],
          'in_reply_to': null,
          'state': {'type': 'editing'},
          'updated_at': 5000,
        };
        return _write('draft', drafts['dr1']);
      case 'compose.update_draft':
        composed.add({'op': r['op'], ...r});
        drafts[r['draft']]!.addAll((r['changes'] as Map).cast());
        return {'type': 'done'};
      case 'compose.attach_file':
        composed.add({'op': r['op'], ...r});
        (drafts[r['draft']]!['attachments'] as List).add({
          'filename': '${r['path']}'.split('/').last,
          'mime_type': 'application/pdf',
          'size': 9,
        });
        return _write('draft', drafts[r['draft']]);
      case 'compose.remove_attachment':
        composed.add({'op': r['op'], ...r});
        (drafts[r['draft']]!['attachments'] as List).removeAt(
          r['position'] as int,
        );
        return {'type': 'done'};
      case 'compose.delete_draft':
        composed.add({'op': r['op'], ...r});
        drafts.remove(r['draft']);
        return {'type': 'done'};
      case 'compose.list_drafts':
        return _write('drafts', [
          for (final d in drafts.values)
            if (d['account'] == r['account']) d,
        ]);
      case 'compose.save_draft_copy':
        composed.add({'op': r['op'], ...r});
        return _write('action', 'copy1');
      case 'compose.send_draft':
        composed.add({'op': r['op'], ...r});
        drafts[r['draft']]!['state'] = {'type': 'sending'};
        return _write('action', 'send1');
      case 'compose.cancel_send':
        composed.add({'op': r['op'], ...r});
        for (final d in drafts.values) {
          d['state'] = {'type': 'editing'};
        }
        return {'type': 'done'};
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

  Map<String, dynamic> _write(String type, Object? data) => {
    'type': 'write',
    'data': {'type': type, 'data': data},
  };

  Map<String, dynamic> _list(String mailbox) {
    final items = switch (mailbox) {
      'mb-inbox' => [
        if (extra) _msg('m9', 'Nouveau', date: 4000),
        _msg('m1', 'Premier', thread: 't1', seen: false, date: 1000),
        _msg('m2', 'Re: Premier', thread: 't1', date: 2000, attachments: true),
        {
          ..._msg('m3', 'Seul', date: 3000),
          'mailboxes': ['mb-inbox', 'mb-other'],
        },
      ],
      'mb-junk' => [_msg('m4', 'Pourriel', date: 500)],
      'mb-drafts' => [
        if (serverCopy)
          {
            ..._msg('m30', 'Salut', date: 5000),
            'message_id_header': 'draft-dr1@mira.local',
          },
      ],
      'mb-other' => [
        if (labelOnly)
          {
            ..._msg('m5', 'Facture', date: 700),
            'mailboxes': ['mb-other'],
          },
      ],
      'mb2-inbox' => [
        {..._msg('m20', 'Chez Pro', date: 600), 'account': 'acc2'},
      ],
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

  group('signature', () {
    Future<MailStore> start() async {
      SharedPreferences.setMockInitialValues({'sig': 'Signature générale'});
      final prefs = await SharedPreferences.getInstance();
      final src = await EngineMailSource.load(FakeEngine());
      return MailStore(prefs, source: src);
    }

    test(
      'un nouveau message porte la signature de l\'adresse d\'envoi',
      () async {
        final store = await start();
        store.openCompose(const ComposeInit());
        expect(store.c!.body.text, '\n\nAlice, Pro');
        expect(store.c!.sigText, 'Alice, Pro');
        store.dispose();
        knownSenders = [];
      },
    );

    test(
      'changer l\'expéditeur remplace la signature, pas le texte écrit',
      () async {
        final store = await start();
        store.openCompose(const ComposeInit());
        final body = store.c!.body;
        body.setContent(
          'Bonjour${body.text}',
          List<int>.filled(7 + body.text.length, 0),
        );
        store.changeFrom('alice@example.org');
        expect(body.text, 'Bonjour\n\nSignature générale');
        expect(store.c!.from, 'alice@example.org');
        expect(store.c!.fromChosen, isTrue);
        store.dispose();
        knownSenders = [];
      },
    );

    test('un message réduit à sa signature compte pour vide', () async {
      final store = await start();
      store.openCompose(const ComposeInit());
      store.c!.dirty = true;
      expect(store.saveDraft(), isFalse);
      store.dispose();
      knownSenders = [];
    });

    test(
      'un transfert place la signature avant le message transféré',
      () async {
        final store = await start();
        store.openMail(store.mails.first.id);
        store.replyTo('fwd');
        final text = store.c!.body.text;
        expect(text.startsWith('\n\n'), isTrue);
        expect(
          text.indexOf('Message transféré'),
          greaterThan(text.indexOf('Signature')),
        );
        expect(store.c!.body.flags.length, text.length);
        store.dispose();
        knownSenders = [];
      },
    );
  });

  test(
    'les libellés sont les boîtes « autre » ; un message en porte',
    () async {
      final src = await EngineMailSource.load(FakeEngine());
      expect(src.labels.keys, ['mb-other']);
      expect(src.labels['mb-other']!.n, 'Factures');
      expect(src.labels['mb-other']!.account, 'acc1');
      final solo = src.boxes['acc1']!.firstWhere((m) => m.subject == 'Seul');
      expect(solo.labels, ['mb-other']);
      final other = src.boxes['acc1']!.firstWhere(
        (m) => m.subject == 'Pourriel',
      );
      expect(other.labels, isEmpty);
    },
  );

  test('poser un libellé envoie add_to_mailbox sur les messages', () async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();
    final fake = FakeEngine();
    final store = MailStore(prefs, source: await EngineMailSource.load(fake));
    final conv = store.mails.firstWhere((m) => m.msgs.length == 2);
    store.sel.add(conv.id);
    store.toggleLabel('mb-other');
    await Future<void>.delayed(Duration.zero);
    expect(conv.labels, ['mb-other']);
    final a = fake.actions.single;
    expect(a['kind'], {'type': 'add_to_mailbox', 'mailbox': 'mb-other'});
    expect(a['targets'], {
      'type': 'messages',
      'messages': ['m1', 'm2'],
    });
    store.toggleLabel('mb-other');
    await Future<void>.delayed(Duration.zero);
    expect(fake.actions.last['kind']['type'], 'remove_from_mailbox');
    store.dispose();
  });

  test('créer, renommer, supprimer un libellé passe par le moteur', () async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();
    final fake = FakeEngine();
    final store = MailStore(prefs, source: await EngineMailSource.load(fake));
    expect(await store.createLabel('acc1', ' Projets '), isTrue);
    expect(fake.actions.last['name'], 'Projets');
    expect(await store.renameLabel('mb-other', 'Compta'), isTrue);
    expect(fake.actions.last['mailbox'], 'mb-other');
    expect(await store.deleteLabel('mb-other'), isTrue);
    expect(fake.actions.last['op'], 'delete_mailbox');
    store.dispose();
  });

  test(
    'deux comptes : libellés séparés, nommés par compte en vue unifiée',
    () async {
      SharedPreferences.setMockInitialValues({'account': 'all'});
      final prefs = await SharedPreferences.getInstance();
      final fake = FakeEngine()..second = true;
      final store = MailStore(prefs, source: await EngineMailSource.load(fake));
      expect(store.labelDefs.keys, unorderedEquals(['mb-other', 'mb2-other']));
      expect(store.labelTitle('mb-other'), 'Factures · Alice');
      expect(store.labelTitle('mb2-other'), 'Factures · Pro');
      final mine = store.mails.firstWhere((m) => m.subject == 'Seul');
      final pro = store.mails.firstWhere((m) => m.subject == 'Chez Pro');
      store.sel.add(mine.id);
      expect(store.labelsForTargets().keys, ['mb-other']);
      store.sel.addAll([mine.id, pro.id]);
      store.toggleLabel('mb2-other');
      await Future<void>.delayed(Duration.zero);
      expect(mine.labels, ['mb-other']);
      expect(pro.labels, ['mb2-other']);
      expect(fake.actions.single['targets']['messages'], ['m20']);
      store.dispose();
    },
  );

  test(
    'un message présent seulement dans un libellé est classé dans Archive',
    () async {
      final fake = FakeEngine()..labelOnly = true;
      final src = await EngineMailSource.load(fake);
      final facture = src.boxes['acc1']!.firstWhere(
        (m) => m.subject == 'Facture',
      );
      expect(facture.folder, 'archive');
      expect(facture.labels, ['mb-other']);
      final seul = src.boxes['acc1']!.firstWhere((m) => m.subject == 'Seul');
      expect(seul.folder, 'inbox');
    },
  );

  test(
    'une boîte du serveur rattachée à un dossier y affiche ses messages',
    () async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final fake = FakeEngine()..labelOnly = true;
      final store = MailStore(prefs, source: await EngineMailSource.load(fake));
      expect(
        store.mails.firstWhere((m) => m.subject == 'Facture').folder,
        'archive',
      );
      await store.mapFolder('mb-other', 'sent');
      final facture = store.mails.firstWhere((m) => m.subject == 'Facture');
      expect(facture.folder, 'sent');
      expect(facture.labels, isEmpty);
      expect(store.labelDefs, isEmpty);
      expect(store.source!.mappable.keys, ['mb-other']);
      expect(prefs.getString('folderMap'), '{"mb-other":"sent"}');
      expect(
        store.mails.firstWhere((m) => m.subject == 'Seul').folder,
        'inbox',
      );
      await store.mapFolder('mb-other', null);
      expect(
        store.mails.firstWhere((m) => m.subject == 'Facture').folder,
        'archive',
      );
      expect(store.labelDefs.keys, ['mb-other']);
      expect(prefs.getString('folderMap'), '{}');
      store.dispose();
    },
  );

  test(
    'les rattachements enregistrés servent dès le premier chargement',
    () async {
      final src = await EngineMailSource.load(
        FakeEngine()..labelOnly = true,
        folderMap: decodeFolderMap('{"mb-other":"sent","x":"nimporte"}'),
      );
      expect(src.folderMap, {'mb-other': 'sent'});
      expect(src.boxes['acc1']!.any((m) => m.folder == 'sent'), isTrue);
      expect(src.labels, isEmpty);
    },
  );

  group('rédaction par le moteur', () {
    Future<(MailStore, FakeEngine)> start() async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final fake = FakeEngine();
      final src = await EngineMailSource.load(fake);
      final store = MailStore(prefs, source: src);
      store.openCompose(
        const ComposeInit(
          to: [Person('Bob', 'bob@example.org')],
          subject: 'Salut',
          text: 'Bonjour',
          from: 'alice@example.org',
        ),
      );
      return (store, fake);
    }

    Map<String, dynamic> last(FakeEngine f, String op) =>
        f.composed.lastWhere((c) => c['op'] == op);

    test(
      'l\'enregistrement crée puis met à jour le brouillon du moteur',
      () async {
        final (store, fake) = await start();
        store.c!.dirty = true;
        store.saveDraft();
        await store.c!.push;
        expect(fake.composed.map((c) => c['op']), [
          'create_draft',
          'update_draft',
        ]);
        expect(last(fake, 'create_draft')['identity'], 'i1');
        final changes = last(fake, 'update_draft')['changes'] as Map;
        expect(changes['subject'], 'Salut');
        expect(changes['body_text'], 'Bonjour');
        expect(changes['to'], [
          {'name': 'Bob', 'address': 'bob@example.org'},
        ]);
        expect(store.c!.id, 'draft:dr1');
        expect(
          store.mails.where((m) => m.folder == 'drafts').single.id,
          'draft:dr1',
        );
        store.dispose();
      },
    );

    test('choisir l\'expéditeur suffit à enregistrer un brouillon', () async {
      final (store, fake) = await start();
      store.c!
        ..to = []
        ..subject.text = ''
        ..body.setContent('', const [])
        ..fromChosen = true;
      store.touch();
      store.saveDraft();
      await store.c!.push;
      expect(fake.composed.map((c) => c['op']), contains('create_draft'));
      store.dispose();
    });

    test('quitter la rédaction copie le brouillon sur le serveur', () async {
      final (store, fake) = await start();
      store.c!.dirty = true;
      store.closeCompose();
      await Future<void>.delayed(const Duration(milliseconds: 50));
      expect(last(fake, 'save_draft_copy')['draft'], 'dr1');
      expect(store.mails.where((m) => m.folder == 'drafts').length, 1);
      store.dispose();
    });

    test('la copie serveur du brouillon n\'apparaît pas en double', () async {
      final (store, fake) = await start();
      store.c!.dirty = true;
      store.saveDraft();
      await store.c!.push;
      fake.serverCopy = true;
      final fresh = await store.source!.reload(store.accounts.single);
      expect(fresh.where((m) => m.folder == 'drafts').length, 1);
      store.dispose();
    });

    test('un second enregistrement ne recrée pas le brouillon', () async {
      final (store, fake) = await start();
      for (var i = 0; i < 2; i++) {
        store.c!.dirty = true;
        store.saveDraft();
      }
      await store.c!.push;
      expect(fake.composed.where((c) => c['op'] == 'create_draft').length, 1);
      expect(fake.composed.where((c) => c['op'] == 'update_draft').length, 2);
      store.dispose();
    });

    test('envoyer met le brouillon en file puis ferme la rédaction', () async {
      final (store, fake) = await start();
      await store.send();
      expect(last(fake, 'send_draft')['draft'], 'dr1');
      expect(last(fake, 'send_draft')['at'], isNull);
      expect(store.c, isNull);
      expect(store.mails.where((m) => m.folder == 'drafts'), isEmpty);
      expect(
        store.mails.where((m) => m.folder == 'sent').map((m) => m.subject),
        contains('Salut'),
      );
      expect(store.toasts.last.label, isNotNull);
      store.dispose();
    });

    test('programmer envoie une date', () async {
      final (store, fake) = await start();
      await store.send('demain 08:00');
      expect(last(fake, 'send_draft')['at'], isA<int>());
      store.dispose();
    });

    test('annuler l\'envoi rouvre le brouillon', () async {
      final (store, fake) = await start();
      await store.send();
      store.toasts.last.cb!();
      await Future<void>.delayed(const Duration(milliseconds: 50));
      expect(last(fake, 'cancel_send')['action'], 'send1');
      expect(store.c?.engineId, 'dr1');
      expect(store.mails.where((m) => m.id == 'sending:dr1'), isEmpty);
      store.dispose();
    });

    test(
      'les pièces jointes locales sont envoyées au moteur, puis retirées',
      () async {
        final (store, fake) = await start();
        final a = Attachment('devis.pdf', 'PDF', '9', path: '/tmp/devis.pdf');
        store.c!.atts.add(a);
        store.c!.dirty = true;
        store.saveDraft();
        await store.c!.push;
        expect(last(fake, 'attach_file')['path'], '/tmp/devis.pdf');
        store.c!.atts.remove(a);
        store.c!.dirty = true;
        store.saveDraft();
        await store.c!.push;
        expect(last(fake, 'remove_attachment')['position'], 0);
        store.dispose();
      },
    );

    test('quitter la rédaction copie le brouillon sur le serveur', () async {
      final (store, fake) = await start();
      store.c!.dirty = true;
      final cs = store.c!;
      store.closeCompose();
      await cs.push;
      expect(fake.composed.map((c) => c['op']), [
        'create_draft',
        'update_draft',
        'save_draft_copy',
      ]);
      expect(last(fake, 'save_draft_copy')['draft'], 'dr1');
      expect(store.mails.where((m) => m.folder == 'drafts'), isNotEmpty);
      store.dispose();
    });

    test('abandonner supprime le brouillon du moteur', () async {
      final (store, fake) = await start();
      store.c!.dirty = true;
      store.saveDraft();
      await store.c!.push;
      store.discard();
      await Future<void>.delayed(const Duration(milliseconds: 20));
      expect(last(fake, 'delete_draft')['draft'], 'dr1');
      store.dispose();
    });

    test('les brouillons du moteur reviennent à la relecture', () async {
      final (store, fake) = await start();
      store.c!.dirty = true;
      store.saveDraft();
      await store.c!.push;
      final fresh = await store.source!.reload(store.accounts.single);
      final d = fresh.singleWhere((m) => m.folder == 'drafts');
      expect(d.id, 'draft:dr1');
      expect(d.to.single.email, 'bob@example.org');
      expect(fake.drafts, isNotEmpty);
      store.dispose();
    });
  });

  test('le contenu d\'une pièce jointe est lu une fois puis gardé', () async {
    final fake = FakeEngine();
    final src = await EngineMailSource.load(fake);
    const a = Attachment('devis.pdf', 'PDF', '2 Ko', id: 'a1');
    expect(await src.attachmentBytes(a), [1, 2, 3]);
    await src.attachmentBytes(a);
    expect(fake.calls.where((c) => c == 'message.save_attachment').length, 1);
    expect(
      await src.attachmentBytes(const Attachment('x', 'PDF', '1')),
      isNull,
    );
  });
}
