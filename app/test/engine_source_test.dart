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
  bool failActions = false;
  String outboxState = 'pending';
  final searches = <String>[];
  final limits = <int>[];
  bool many = false;
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
            {'id': 'mb-inbox', 'role': 'inbox', 'unread': 4321},
            {'id': 'mb-junk', 'role': 'junk'},
            {'id': 'mb-drafts', 'role': 'drafts'},
            {'id': 'mb-archive', 'role': 'archive'},
            {
              'id': 'mb-other',
              'role': 'other',
              'name': 'Factures',
              'display_name': 'Factures',
            },
          ],
        };
      case 'message.list_messages':
        final limit = ((r['page'] as Map)['limit'] as num).toInt();
        limits.add(limit);
        if (many && (r['view'] as Map)['mailbox'] == 'mb-inbox') {
          final all = [
            for (var i = 0; i < 5; i++) _msg('p$i', 'Ancien $i', date: 900 - i),
          ];
          return {
            'type': 'messages',
            'data': {
              'items': all.take(limit).toList(),
              'next_cursor': limit < all.length ? 'c' : null,
            },
          };
        }
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
          'action.empty_mailbox' ||
          'action.create_mailbox' ||
          'action.rename_mailbox' ||
          'action.delete_mailbox':
        if (failActions) {
          throw const EngineException('internal.bug', 'refusé');
        }
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
      case 'search.search':
        searches.add('${r['query']}');
        return {
          'type': 'messages',
          'data': {
            'items': [
              {
                ..._msg('m50', 'Contrat ancien', date: 100),
                'mailboxes': ['mb-archive'],
              },
              {
                ..._msg('m1', 'Premier', thread: 't1', seen: false, date: 1000),
                'mailboxes': ['mb-inbox'],
              },
            ],
            'next_cursor': null,
          },
        };
      case 'search.search_server':
        searches.add('server:${r['query']}');
        return {
          'type': 'operation',
          'data': {
            'id': 'op-s',
            'kind': 'search_server',
            'status': {'type': 'running', 'done': 0, 'total': 1},
          },
        };
      case 'maintenance.get_storage_stats':
        return {
          'type': 'storage_stats',
          'data': {
            'messages': 12,
            'attachments': 3,
            'attachments_cached': 1,
            'raw_bytes': 5000,
            'compressed_bytes': 2048,
          },
        };
      case 'maintenance.check_integrity':
        return {
          'type': 'integrity',
          'data': {'ok': true},
        };
      case 'maintenance.purge_attachment_cache' ||
          'maintenance.rebuild_search_index':
        actions.add({'op': r['op'], ...r});
        return {'type': 'ok'};
      case 'maintenance.export_diagnostic':
        actions.add({'op': r['op'], ...r});
        return {
          'type': 'exported',
          'data': {'path': r['destination'], 'bytes': 321},
        };
      case 'sync.extend_window':
        actions.add({'op': r['op'], ...r});
        return {
          'type': 'operation',
          'data': {
            'id': 'op-x',
            'kind': 'extend_window',
            'status': {'type': 'running', 'done': 0, 'total': 1},
          },
        };
      case 'operation.cancel_operation':
        actions.add({'op': r['op'], ...r});
        return {'type': 'ok'};
      case 'operation.get_operation':
        return {
          'type': 'operation',
          'data': {
            'id': 'op-s',
            'kind': 'search_server',
            'status': {'type': 'finished'},
          },
        };
      case 'search.get_server_search_result':
        return {
          'type': 'server_search',
          'data': {
            'hits': [
              {'mailbox': 'mb-archive', 'uid': 5, 'message': 'm60'},
              {'mailbox': 'mb-archive', 'uid': 6, 'message': null},
            ],
            'truncated': false,
          },
        };
      case 'message.get_message':
        return {
          'type': 'message',
          'data': {
            ..._msg('m60', 'Corps ancien', date: 50),
            'mailboxes': ['mb-archive'],
          },
        };
      case 'compose.list_outbox':
        return _write('outbox', [
          for (final d in drafts.values)
            if ((d['state'] as Map)['type'] != 'editing')
              {
                'action': 'send1',
                'draft': d['id'],
                'state': outboxState,
                'scheduled_for': 9000,
                'subject': d['subject'],
                'last_error': outboxState == 'failed'
                    ? 'serveur injoignable'
                    : null,
              },
        ]);
      case 'queue.retry_action' || 'compose.resolve_uncertain_send':
        composed.add({'op': r['op'], ...r});
        return {'type': 'done'};
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
  test('entretien : volumes, intégrité, purge, index et export', () async {
    final fake = FakeEngine();
    final admin = (await EngineMailSource.load(fake)).maintenance;
    final st = await admin.stats();
    expect(
      (st.messages, st.attachments, st.cached, st.compressedBytes),
      (12, 3, 1, 2048),
    );
    expect(await admin.checkIntegrity(), isTrue);
    await admin.purgeAttachments();
    await admin.rebuildIndex();
    expect(await admin.export('export_diagnostic', '/tmp/d.json'), 321);
    expect(fake.actions.map((a) => a['op']), [
      'purge_attachment_cache',
      'rebuild_search_index',
      'export_diagnostic',
    ]);
    expect(fake.actions.last['destination'], '/tmp/d.json');
  });

  test(
    'hors ligne, échec et avancement de la synchro suivent les événements',
    () async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final fake = FakeEngine();
      final store = MailStore(prefs, source: await EngineMailSource.load(fake));
      Future<void> emit(Map<String, dynamic> kind) async {
        fake.feed.add({'kind': kind});
        await Future<void>.delayed(const Duration(milliseconds: 10));
      }

      expect(store.syncIssues, isEmpty);
      await emit({'type': 'went_offline', 'account': 'acc1'});
      expect(store.syncIssues.single.$2.offline, isTrue);
      await emit({'type': 'back_online', 'account': 'acc1'});
      expect(store.syncIssues, isEmpty);
      await emit({
        'type': 'sync_progress',
        'account': 'acc1',
        'done': 1,
        'total': 4,
      });
      expect(store.syncIssues.single.$2.percent, 25);
      await emit({
        'type': 'sync_failed',
        'account': 'acc1',
        'code': 'network.timeout',
      });
      expect(store.syncIssues.single.$2.failed, 'network.timeout');
      await emit({'type': 'sync_finished', 'account': 'acc1'});
      expect(store.syncIssues, isEmpty);
      store.dispose();
    },
  );

  test(
    'le compteur de non lus vient du moteur, pas des messages chargés',
    () async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final src = await EngineMailSource.load(FakeEngine());
      expect(src.unreadOf('acc1', 'inbox'), 4321);
      final store = MailStore(prefs, source: src);
      expect(store.navCount('inbox'), 4321);
      store.dispose();
    },
  );

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

  test(
    'télécharger enregistre le contenu de la pièce jointe du moteur',
    () async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final store = MailStore(
        prefs,
        source: await EngineMailSource.load(FakeEngine()),
      );
      String? name;
      List<int>? saved;
      store.saveFile = (n, b) async {
        name = n;
        saved = b;
        return Uri.file('/tmp/$n');
      };
      await store.download(
        const Attachment('devis.pdf', 'PDF', '2 Ko', id: 'a1'),
      );
      expect(name, 'devis.pdf');
      expect(saved, [1, 2, 3]);
      expect(store.toasts.last.err, isFalse);
      store.saveFile = (_, _) async => null;
      await store.download(
        const Attachment('devis.pdf', 'PDF', '2 Ko', id: 'a1'),
      );
      store.saveFile = (_, _) => throw StateError('refusé');
      await store.download(
        const Attachment('devis.pdf', 'PDF', '2 Ko', id: 'a1'),
      );
      expect(store.toasts.last.err, isTrue);
      store.dispose();
    },
  );

  test(
    'messages plus anciens : une page de plus à chaque demande, jusqu\'au bout',
    () async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final fake = FakeEngine()..many = true;
      final src = await EngineMailSource.load(fake, pageSize: 2);
      final store = MailStore(prefs, source: src);
      int ancients() =>
          store.mails.where((m) => m.subject.startsWith('Ancien')).length;
      expect(ancients(), 2);
      expect(store.moreAvailable, isTrue);
      await store.loadMore();
      expect(ancients(), 4);
      expect(store.moreAvailable, isTrue);
      await store.loadMore();
      expect(ancients(), 5);
      expect(store.moreAvailable, isFalse);
      final calls = fake.limits.length;
      await store.loadMore();
      expect(fake.limits.length, calls);
      store.dispose();
    },
  );

  group('reports', () {
    Future<(MailStore, SharedPreferences)> start() async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final src = await EngineMailSource.load(FakeEngine());
      return (MailStore(prefs, source: src), prefs);
    }

    test(
      'un message reporté reste reporté après relecture et redémarrage',
      () async {
        final (store, prefs) = await start();
        final conv = store.mails.firstWhere((m) => m.msgs.length == 2);
        store.sel.add(conv.id);
        store.snooze('tom');
        expect(conv.folder, 'snoozed');
        await store.source!.reload(store.accounts.single);
        final fresh = await store.source!.reload(store.accounts.single);
        expect(fresh.firstWhere((m) => m.id == conv.id).folder, 'inbox');
        store.dispose();
        final again = MailStore(
          prefs,
          source: await EngineMailSource.load(FakeEngine()),
        );
        expect(
          again.mails.firstWhere((m) => m.id == conv.id).folder,
          'snoozed',
        );
        expect(
          again.mails.firstWhere((m) => m.id == conv.id).snooze,
          'demain 08:00',
        );
        again.dispose();
      },
    );

    test(
      'à l\'heure du rappel le message revient en réception, non lu',
      () async {
        SharedPreferences.setMockInitialValues({
          'snoozes': '{"acc1/inbox/t1":[1,"demain 08:00"]}',
        });
        final prefs = await SharedPreferences.getInstance();
        final store = MailStore(
          prefs,
          source: await EngineMailSource.load(FakeEngine()),
        );
        final conv = store.mails.firstWhere((m) => m.id == 'acc1/inbox/t1');
        expect(conv.folder, 'inbox');
        expect(conv.unread, isTrue);
        expect(prefs.getString('snoozes'), '{}');
        expect(store.toasts.last.msg, contains('Rappel'));
        store.dispose();
      },
    );

    test('annuler un report l\'oublie', () async {
      final (store, prefs) = await start();
      final conv = store.mails.firstWhere((m) => m.msgs.length == 2);
      store.sel.add(conv.id);
      store.snooze('1h');
      store.toasts.last.cb!();
      expect(conv.folder, 'inbox');
      expect(prefs.getString('snoozes'), '{}');
      store.dispose();
    });
  });

  group('recherche', () {
    Future<(MailStore, FakeEngine)> start() async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final fake = FakeEngine();
      return (
        MailStore(prefs, source: await EngineMailSource.load(fake)),
        fake,
      );
    }

    Future<void> wait() =>
        Future<void>.delayed(const Duration(milliseconds: 400));

    test('la recherche interroge l\'index du moteur et trouve des messages non chargés', () async {
      final (store, fake) = await start();
      store.setQuery('contrat');
      await wait();
      expect(fake.searches, ['contrat']);
      final subjects = store.visible().map((m) => m.subject);
      expect(subjects, contains('Contrat ancien'));
      expect(
        store.visible().firstWhere((m) => m.subject == 'Contrat ancien').folder,
        'archive',
      );
      store.dispose();
    });

    test(
      'un message déjà chargé n\'apparaît qu\'une fois et garde son état',
      () async {
        final (store, _) = await start();
        store.setQuery('premier');
        await wait();
        final hits = store.visible().where(
          (m) => m.subject.contains('Premier'),
        );
        expect(hits.length, 1);
        expect(
          identical(
            hits.single,
            store.mails.firstWhere((m) => m.msgs.length == 2),
          ),
          isTrue,
        );
        store.dispose();
      },
    );

    test('une saisie trop courte ne lance pas de recherche, effacer la requête rend la liste', () async {
      final (store, fake) = await start();
      store.setQuery('c');
      await wait();
      expect(fake.searches, isEmpty);
      store.setQuery('contrat');
      await wait();
      store.setQuery('');
      expect(store.visible().every((m) => m.folder == store.folder), isTrue);
      store.dispose();
    });

    test(
      'chercher sur le serveur ajoute les messages connus et compte les autres',
      () async {
        final (store, fake) = await start();
        store.setQuery('contrat');
        await wait();
        await store.searchServer();
        expect(fake.searches, contains('server:contrat'));
        expect(store.visible().map((m) => m.subject), contains('Corps ancien'));
        expect(store.serverNote, contains('1'));
        expect(store.serverSearching, isFalse);
        store.setQuery('autre');
        expect(store.serverNote, isNull);
        store.dispose();
      },
    );

    test('un résultat trouvé par le moteur peut être ouvert', () async {
      final (store, _) = await start();
      store.setQuery('contrat');
      await wait();
      final hit = store.visible().firstWhere(
        (m) => m.subject == 'Contrat ancien',
      );
      store.openMail(hit.id);
      expect(store.open, hit.id);
      store.dispose();
    });
  });

  group('boîte d\'envoi', () {
    Future<(MailStore, FakeEngine)> sent({String? sched}) async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final fake = FakeEngine();
      final store = MailStore(prefs, source: await EngineMailSource.load(fake));
      store.openCompose(
        const ComposeInit(
          to: [Person('Bob', 'bob@example.org')],
          subject: 'Programmé',
          text: 'Bonjour',
          from: 'alice@example.org',
        ),
      );
      await store.send(sched);
      await Future<void>.delayed(const Duration(milliseconds: 30));
      return (store, fake);
    }

    test('un envoi programmé apparaît dans la boîte d\'envoi', () async {
      final (store, _) = await sent(sched: 'demain 08:00');
      final m = store.mails.singleWhere((x) => x.folder == 'outbox');
      expect(m.id, 'outbox:dr1');
      expect(m.subject, 'Programmé');
      expect(m.outbox!.waiting, isTrue);
      expect(store.navCount('outbox'), 1);
      expect(store.targets(), isEmpty);
      store.dispose();
    });

    test(
      'un envoi immédiat n\'est pas montré en double pendant le délai',
      () async {
        final (store, _) = await sent();
        expect(store.mails.where((x) => x.folder == 'outbox'), isEmpty);
        expect(store.mails.where((x) => x.folder == 'sent'), isNotEmpty);
        store.dispose();
      },
    );

    test('annuler un envoi le ramène dans les brouillons', () async {
      final (store, fake) = await sent(sched: 'demain 08:00');
      await store.cancelOutbox(
        store.mails.singleWhere((x) => x.folder == 'outbox'),
      );
      expect(fake.composed.last['op'], 'cancel_send');
      expect(store.mails.where((x) => x.folder == 'outbox'), isEmpty);
      expect(store.mails.where((x) => x.folder == 'drafts'), isNotEmpty);
      store.dispose();
    });

    test('un envoi échoué se relance, un envoi incertain se tranche', () async {
      final (store, fake) = await sent(sched: 'demain 08:00');
      fake.outboxState = 'failed';
      await store.source!.reload(store.accounts.single);
      await store.retryOutbox(
        (await store.source!.reload(store.accounts.single))
            .singleWhere((x) => x.folder == 'outbox'),
      );
      expect(fake.composed.last['op'], 'retry_action');
      fake.outboxState = 'uncertain';
      final uncertain = (await store.source!.reload(store.accounts.single))
          .singleWhere((x) => x.folder == 'outbox');
      expect(uncertain.outbox!.state, 'uncertain');
      await store.resolveOutbox(uncertain, resend: true);
      expect(fake.composed.last['resend'], isTrue);
      store.dispose();
    });
  });

  group('actions sur les messages', () {
    Future<(MailStore, FakeEngine, Mail)> start() async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final fake = FakeEngine();
      final store = MailStore(prefs, source: await EngineMailSource.load(fake));
      final conv = store.mails.firstWhere((m) => m.msgs.length == 2);
      return (store, fake, conv);
    }

    Future<void> settle() =>
        Future<void>.delayed(const Duration(milliseconds: 20));

    test(
      'archiver envoie archive, annuler déplace vers la réception',
      () async {
        final (store, fake, conv) = await start();
        store.sel.add(conv.id);
        store.archive();
        await settle();
        expect(fake.actions.last['kind'], {'type': 'archive'});
        expect(fake.actions.last['targets']['messages'], ['m1', 'm2']);
        store.toasts.last.cb!();
        await settle();
        expect(fake.actions.last['kind'], {'type': 'move', 'to': 'mb-inbox'});
        expect(conv.folder, 'inbox');
        store.dispose();
      },
    );

    testWidgets('vider le spam appelle empty_mailbox sur la boîte entière', (
      tester,
    ) async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final fake = FakeEngine();
      final src = await tester.runAsync(() => EngineMailSource.load(fake));
      final store = MailStore(prefs, source: src);
      store.selectFolder('spam');
      store.emptyFolder();
      expect(store.mails.where((m) => m.folder == 'spam'), isEmpty);
      await tester.pump(const Duration(seconds: 7));
      await tester.runAsync(() => Future<void>.delayed(Duration.zero));
      expect(fake.actions.last['op'], 'empty_mailbox');
      expect(fake.actions.last['mailbox'], 'mb-junk');
      store.dispose();
    });

    test('corbeille et spam passent par le moteur', () async {
      final (store, fake, conv) = await start();
      store.sel.add(conv.id);
      store.trash();
      await settle();
      expect(fake.actions.last['kind'], {'type': 'trash'});
      store.sel.add(conv.id);
      store.spam();
      await settle();
      expect(fake.actions.last['kind'], {'type': 'mark_spam', 'spam': true});
      store.sel.add(conv.id);
      store.spam();
      await settle();
      expect(fake.actions.last['kind'], {'type': 'mark_spam', 'spam': false});
      store.dispose();
    });

    test('favori et lu/non lu passent par le moteur', () async {
      final (store, fake, conv) = await start();
      store.toggleStar(conv.id);
      await settle();
      expect(fake.actions.last['kind'], {
        'type': 'mark_flagged',
        'flagged': true,
      });
      store.sel.add(conv.id);
      store.toggleRead();
      await settle();
      expect(fake.actions.last['kind']['type'], 'mark_read');
      store.dispose();
    });

    test('ouvrir un message non lu le marque lu sur le serveur', () async {
      final (store, fake, conv) = await start();
      expect(conv.unread, isTrue);
      store.openMail(conv.id);
      await settle();
      expect(fake.actions.last['kind'], {'type': 'mark_read', 'read': true});
      store.dispose();
    });

    test(
      'la suppression définitive attend la fin du délai d\'annulation',
      () async {
        final (store, fake, conv) = await start();
        conv.folder = 'trash';
        store.sel.add(conv.id);
        store.trash();
        await settle();
        expect(fake.actions, isEmpty);
        store.toasts.last.cb!();
        await settle();
        expect(fake.actions, isEmpty);
        expect(store.mails, contains(conv));
        store.dispose();
      },
    );

    test('une action refusée relit le compte et prévient', () async {
      final (store, fake, conv) = await start();
      fake.failActions = true;
      store.toggleStar(conv.id);
      await settle();
      expect(store.toasts.any((t) => t.err), isTrue);
      final again = store.mails.firstWhere((m) => m.msgs.length == 2);
      expect(again.starred, isFalse);
      store.dispose();
    });
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
