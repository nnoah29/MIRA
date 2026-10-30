import 'dart:convert';
import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/html_lite.dart';
import 'package:mira/data/models.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/engine_mail_source.dart';
import 'package:mira/state/folder_map.dart';
import 'package:mira/state/mail_notifier.dart';
import 'package:mira/state/store.dart';
import 'package:shared_preferences/shared_preferences.dart';

class FakeEngine implements EngineClient {
  final calls = <String>[];
  final queueAsked = <Map<String, dynamic>>[];
  final filteredAsked = <Map<String, dynamic>>[];
  final bodyFails = <String>{};
  final noText = <String>{};
  bool fetchFails = false;
  bool allowed = false;
  bool extra = false;
  bool arrived = false;
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
  final extraInbox = <Map<String, dynamic>>[];
  final localLabels = <Map<String, dynamic>>[];
  final localPoses = <Map<String, dynamic>>[];
  final systemPoses = <Map<String, dynamic>>[];
  final labelWrites = <Map<String, dynamic>>[];
  bool importFails = false;
  bool hideInbox = false;
  bool settingsOn = false;
  int? enginePoll;
  List<String> engineMuted = [];
  bool noAccounts = false;
  Map<String, dynamic>? engineQuiet;
  bool engineSettingsOn = false;
  final settingWrites = <Map<String, dynamic>>[];
  bool hideRow = false;
  final labelViews = <Map<String, dynamic>>[];

  Map<String, dynamic> _settingsAnswer() => {
    'type': 'settings',
    'data': {
      'type': 'settings',
      'data': {
        'poll_secs': enginePoll,
        'muted_accounts': engineMuted,
        'quiet_hours': engineQuiet,
      },
    },
  };

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
            if (!noAccounts)
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
        final view = r['view'] as Map;
        final filters = r['filters'] as Map;
        if (view['type'] == 'local_label') {
          labelViews.add({...view});
          return {
            'type': 'messages',
            'data': {
              'items': [
                _msg('h1', 'Masqué', thread: 'th1', date: 800)
                  ..['mailboxes'] = ['mb-inbox'],
              ],
              'next_cursor': null,
            },
          };
        }
        if (view['type'] == 'standard' || filters['unread_only'] == true) {
          filteredAsked.add(r);
          final deep = view['type'] == 'standard'
              ? (_msg('fl1', 'Vieux favori', date: 3)
                  ..['flags'] = {'seen': true, 'flagged': true}
                  ..['mailboxes'] = ['mb-archive'])
              : _msg('nr1', 'Vieux non lu', seen: false, date: 5);
          return {
            'type': 'messages',
            'data': {
              'items': [deep],
              'next_cursor': null,
            },
          };
        }
        if (many && view['mailbox'] == 'mb-inbox') {
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
            'text': noText.contains(r['message']) ? null : 'Un\n\nDeux',
            'html_available': r['message'] == 'm2',
          },
        };
      case 'message.fetch_message_body':
        if (fetchFails) {
          throw const EngineException('network.down', 'hors ligne');
        }
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
      case 'sync.get_sync_status':
        return {
          'type': 'sync',
          'data': {
            'type': 'status',
            'data': [
              {
                'account': 'acc1',
                'last_error': 'auth.failed',
                'last_synced_at': 1700000000000,
                'mailboxes_in_progress': [],
                'pending_actions': 0,
                'state': {'type': 'connected'},
                'syncing': false,
              },
            ],
          },
        };
      case 'queue.list_actions':
        queueAsked.add(r);
        return {
          'type': 'write',
          'data': {
            'type': 'queued_actions',
            'data': [
              {
                'id': 'q1',
                'kind': {'type': 'archive'},
                'state': 'failed',
                'targets': ['m1', 'm2'],
                'attempts': 3,
                'last_error': 'network.timeout',
              },
              {
                'id': 'q2',
                'kind': {'type': 'send', 'draft': 'dr1'},
                'state': 'pending',
                'targets': [],
                'attempts': 0,
                'last_error': null,
              },
            ],
          },
        };
      case 'queue.cancel_action':
        actions.add({'op': r['op'], ...r});
        return {'type': 'done'};
      case 'mailbox.set_synced_mailboxes':
        actions.add({'op': r['op'], ...r});
        return {'type': 'done'};
      case 'message.get_raw_source':
        return {
          'type': 'raw_source',
          'data': {
            'size': 20,
            'content_base64': base64Encode(
              utf8.encode('Subject: Été\r\n\r\nok'),
            ),
          },
        };
      case 'sync.resync_mailbox':
        actions.add({'op': r['op'], ...r});
        return {'type': 'done'};
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
      case 'label_settings.list_labels':
        if (!settingsOn) throw EngineException('internal.bug', 'absent');
        return {
          'type': 'label_settings',
          'data': {
            'type': 'labels',
            'data': [
              {
                'label': {'kind': 'local', 'id': 'l1'},
                'account': null,
                'name': 'À payer',
                'color': 1,
                'messages': 1,
                'unused': false,
                'settings': {
                  'hide_from_inbox': hideInbox || hideRow,
                  'counter': 'total',
                },
              },
            ],
          },
        };
      case 'label_settings.set_label_settings':
        hideInbox = r['hide_from_inbox'] == true;
        return {
          'type': 'label_settings',
          'data': {'type': 'done', 'data': null},
        };
      case 'settings.get_settings':
        if (!engineSettingsOn) throw EngineException('internal.bug', 'absent');
        return _settingsAnswer();
      case 'settings.set_settings':
        final given = (r['settings'] as Map).cast<String, dynamic>();
        settingWrites.add(r);
        final before = _settingsAnswer().toString();
        enginePoll = given['poll_secs'] as int?;
        engineMuted = [...(given['muted_accounts'] as List).cast<String>()];
        engineQuiet = (given['quiet_hours'] as Map?)?.cast<String, dynamic>();
        if (_settingsAnswer().toString() != before) {
          feed.add({
            'kind': {
              'type': 'settings_changed',
              'keys': ['poll_secs'],
            },
          });
        }
        return _settingsAnswer();
      case 'local_label.list_local_labels':
        return _labelAnswer('labels', localLabels);
      case 'system_label.list_system_label_assignments':
        return {
          'type': 'system_label',
          'data': {'type': 'assignments', 'data': systemPoses},
        };
      case 'local_label.list_local_label_assignments':
        return _labelAnswer('assignments', localPoses);
      case 'local_label.import_local_labels':
        if (importFails) throw EngineException('internal.bug', 'indisponible');
        labelWrites.add(r);
        return _labelAnswer('imported', {
          'labels': 1,
          'assignments': 0,
          'ignored': 0,
        });
      case 'local_label.create_local_label':
        labelWrites.add(r);
        return _labelAnswer('label', {
          'id': 'l-new',
          'name': r['name'],
          'color': r['color'],
        });
      case 'local_label.set_local_label' ||
          'local_label.rename_local_label' ||
          'local_label.recolor_local_label' ||
          'local_label.delete_local_label':
        labelWrites.add(r);
        return _labelAnswer('done', null);
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

  Map<String, dynamic> _labelAnswer(String type, Object? data) => {
    'type': 'local_label',
    'data': {'type': type, 'data': data},
  };

  Map<String, dynamic> _write(String type, Object? data) => {
    'type': 'write',
    'data': {'type': type, 'data': data},
  };

  Map<String, dynamic> _list(String mailbox) {
    final items = switch (mailbox) {
      'mb-inbox' => [
        ...extraInbox,
        if (extra) _msg('m9', 'Nouveau', date: 4000),
        if (arrived)
          _msg(
            'm10',
            'Arrivé à l’instant',
            seen: false,
            date: DateTime.now().millisecondsSinceEpoch,
          ),
        _msg('m1', 'Premier', thread: 't1', seen: false, date: 1000),
        _msg('m2', 'Re: Premier', thread: 't1', date: 2000, attachments: true),
        if (!hideInbox)
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

class FakeNotifications implements SystemNotifications {
  final shown = <(String, String, String?)>[];
  void Function(String)? tap;

  @override
  Future<void> init(void Function(String payload) onTap) async => tap = onTap;

  @override
  Future<void> show(
    int id,
    String title,
    String body, {
    String? payload,
    bool sound = true,
  }) async => shown.add((title, body, payload));
}

void main() {
  group('notifications', () {
    Future<(MailStore, FakeEngine, FakeNotifications)> start([
      Map<String, Object> prefs = const {},
    ]) async {
      SharedPreferences.setMockInitialValues(prefs);
      final p = await SharedPreferences.getInstance();
      final fake = FakeEngine();
      final store = MailStore(p, source: await EngineMailSource.load(fake));
      final n = FakeNotifications();
      await store.useNotifications(n);
      store.appFocused = false;
      return (store, fake, n);
    }

    Future<void> arrive(FakeEngine fake) async {
      fake.arrived = true;
      fake.feed.add({
        'kind': {
          'type': 'new_message_received',
          'account': 'acc1',
          'message': 'm10',
          'mailbox': 'mb-inbox',
        },
      });
      await Future<void>.delayed(const Duration(milliseconds: 400));
    }

    test(
      'un nouveau message notifie expéditeur et objet ; le clic l’ouvre',
      () async {
        final (store, fake, n) = await start();
        await arrive(fake);
        expect(n.shown.single.$1, 'Bob');
        expect(n.shown.single.$2, 'Arrivé à l’instant');
        n.tap!(n.shown.single.$3!);
        expect(store.open, n.shown.single.$3);
        store.dispose();
      },
    );

    test(
      'rien au premier plan, en mode discret le contenu est masqué',
      () async {
        final (store, fake, n) = await start({'notifyPreview': '0'});
        store.appFocused = true;
        await arrive(fake);
        expect(n.shown, isEmpty);
        store.setNotify('notifyFocused', '1');
        fake.arrived = false;
        await store.refreshNow();
        await arrive(fake);
        expect(n.shown.single.$1, 'Nouveau message');
        expect(n.shown.single.$2, 'alice@example.org');
        store.dispose();
      },
    );

    void verdict(FakeEngine fake, String id, bool silenced) => fake.feed.add({
      'kind': {
        'type': 'new_message_received',
        'account': 'acc1',
        'message': id,
        'mailbox': 'mb-inbox',
        'silenced': silenced,
      },
    });

    void touch(FakeEngine fake) => fake.feed.add({
      'kind': {'type': 'message_updated', 'account': 'acc1', 'message': 'm1'},
    });

    Map<String, dynamic> recent(
      FakeEngine fake,
      String id,
      String thread,
      int agoMs, {
      bool seen = false,
    }) => fake._msg(
      id,
      'Sujet $id',
      thread: thread,
      seen: seen,
      date: DateTime.now().millisecondsSinceEpoch - agoMs,
    );

    Future<void> settle() =>
        Future<void>.delayed(const Duration(milliseconds: 400));

    test(
      'fil avec deux nouveaux messages, le plus récent muet : notifié une fois',
      () async {
        final (store, fake, n) = await start();
        fake.extraInbox.addAll([
          recent(fake, 'm11', 't9', 2000),
          recent(fake, 'm12', 't9', 1000),
        ]);
        verdict(fake, 'm11', false);
        verdict(fake, 'm12', true);
        await settle();
        expect(n.shown, hasLength(1));
        expect(n.shown.single.$3, 'acc1/inbox/t9');
        store.dispose();
      },
    );

    test('un nouveau message muet plus ancien que le dernier du fil n’ajoute aucune notification', () async {
      final (store, fake, n) = await start();
      fake.extraInbox.add(recent(fake, 'm15', 't8', 0));
      verdict(fake, 'm15', false);
      await settle();
      expect(n.shown, hasLength(1));
      fake.extraInbox.add(recent(fake, 'm14', 't8', 5000));
      verdict(fake, 'm14', true);
      await settle();
      expect(n.shown, hasLength(1));
      store.dispose();
    });

    test('verdict reçu après la relecture : muet jamais notifié, non muet notifié une fois', () async {
      final (store, fake, n) = await start();
      fake.extraInbox.add(recent(fake, 'm16', 't7', 0));
      touch(fake);
      await settle();
      expect(n.shown, isEmpty);
      verdict(fake, 'm16', true);
      await settle();
      expect(n.shown, isEmpty);
      fake.extraInbox.add(recent(fake, 'm17', 't6', 0));
      touch(fake);
      await settle();
      expect(n.shown, isEmpty);
      verdict(fake, 'm17', false);
      await settle();
      expect(n.shown.single.$3, 'acc1/inbox/t6');
      store.dispose();
    });

    test('rule_changed du moteur est relayé aux réglages des règles', () async {
      final (store, fake, _) = await start();
      final got = store.source!.ruleChanges.first;
      fake.feed.add({
        'kind': {'type': 'rule_changed', 'rule': 'r1'},
      });
      await got.timeout(const Duration(seconds: 2));
      store.dispose();
    });

    test('compte coupé ou notifications désactivées : rien', () async {
      final (store, fake, n) = await start({'notifyMuted': '["acc1"]'});
      await arrive(fake);
      expect(n.shown, isEmpty);
      store.dispose();
    });

    test('heures calmes, y compris à cheval sur minuit', () {
      SharedPreferences.setMockInitialValues({'notifyQuiet': '22-7'});
      return SharedPreferences.getInstance().then((p) {
        final cfg = NotifySettings(p);
        expect([22, 23, 0, 6].every(cfg.isQuiet), isTrue);
        expect([7, 12, 21].any(cfg.isQuiet), isFalse);
      });
    });

    test('les messages déjà connus ou anciens ne sont pas « nouveaux »', () {
      final now = DateTime.now().millisecondsSinceEpoch;
      Mail mail(String ref, int date) => Mail(
        id: 'a/inbox/$ref',
        folder: 'inbox',
        from: const Person('B', 'b@x.org'),
        to: const [],
        subject: ref,
        date: date,
        unread: true,
        msgs: [
          Msg(
            from: const Person('B', 'b@x.org'),
            date: date,
            body: const [],
            ref: ref,
          ),
        ],
        account: 'a',
      );
      final before = [mail('old', now)];
      expect(newArrivals(null, [mail('x', now)], now), isEmpty);
      expect(
        newArrivals(before, [
          mail('old', now),
          mail('x', now),
          mail('ancient', 0),
        ], now).map((m) => m.subject),
        ['x'],
      );
    });
  });

  group('filtres sur toute la boîte', () {
    Future<(MailStore, FakeEngine)> start() async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final fake = FakeEngine();
      return (
        MailStore(prefs, source: await EngineMailSource.load(fake)),
        fake,
      );
    }

    Future<void> settle() =>
        Future<void>.delayed(const Duration(milliseconds: 20));

    test(
      'Non lus : le moteur trouve un non lu hors des pages chargées',
      () async {
        final (store, fake) = await start();
        store.setOnly('unread');
        await settle();
        final asked = fake.filteredAsked.single;
        expect(asked['view'], {'type': 'mailbox', 'mailbox': 'mb-inbox'});
        expect((asked['filters'] as Map)['unread_only'], isTrue);
        final shown = store.visible();
        expect(shown.every((m) => m.unread), isTrue);
        final old = shown.firstWhere((m) => m.subject == 'Vieux non lu');
        expect(store.byId(old.id), same(old));
        store.setOnly('unread');
        expect(store.only, isNull);
        store.dispose();
      },
    );

    test(
      'Favoris : la vue « flagged » du moteur complète les pages chargées',
      () async {
        final (store, fake) = await start();
        store.selectFolder('starred');
        await settle();
        expect(fake.filteredAsked.single['view'], {
          'type': 'standard',
          'view': 'flagged',
        });
        expect(store.visible().map((m) => m.subject), contains('Vieux favori'));
        store.selectFolder('inbox');
        expect(
          store.visible().map((m) => m.subject),
          isNot(contains('Vieux favori')),
        );
        store.dispose();
      },
    );
  });

  test('état de synchro par compte lu dans le moteur', () async {
    final admin = (await EngineMailSource.load(FakeEngine())).admin;
    final st = (await admin.syncStatus())['acc1']!;
    expect(
      (st.lastSyncedAt, st.lastError, st.syncing),
      (1700000000000, 'auth.failed', false),
    );
  });

  test(
    'actions en attente : envois exclus, abandon envoyé au moteur',
    () async {
      final fake = FakeEngine();
      final admin = (await EngineMailSource.load(fake)).admin;
      final list = await admin.pendingActions('acc1');
      expect(list.single.id, 'q1');
      expect(
        (list.single.kind, list.single.targets, list.single.error),
        ('archive', 2, 'network.timeout'),
      );
      expect(fake.queueAsked.single['states'], contains('failed'));
      await admin.cancelAction('q1');
      expect(fake.actions.last['op'], 'cancel_action');
      expect(fake.actions.last['action'], 'q1');
    },
  );

  test('dossiers synchronisés : lus puis envoyés au moteur', () async {
    final fake = FakeEngine();
    final admin = (await EngineMailSource.load(fake)).admin;
    final boxes = await admin.mailboxes('acc1');
    expect(boxes.first.id, 'mb-inbox');
    expect(boxes.every((b) => b.synced), isTrue);
    await admin.setSyncedMailboxes('acc1', ['mb-inbox', 'mb-drafts']);
    expect(fake.actions.last['op'], 'set_synced_mailboxes');
    expect(fake.actions.last['mailboxes'], ['mb-inbox', 'mb-drafts']);
  });

  test('original du message et resynchronisation du dossier', () async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();
    final fake = FakeEngine();
    final store = MailStore(prefs, source: await EngineMailSource.load(fake));
    final conv = store.mails.firstWhere((m) => m.folder == 'inbox');
    expect(await store.originalOf(conv), 'Subject: Été\r\n\r\nok');
    await store.resyncFolder();
    expect(fake.actions.last['op'], 'resync_mailbox');
    expect(fake.actions.last['mailbox'], 'mb-inbox');
    expect(store.toasts.last.msg, 'Resynchronisation lancée');
    store.dispose();
  });

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

  test(
    'les marqueurs système du moteur deviennent des marqueurs de conversation',
    () async {
      final fake = FakeEngine();
      fake.extraInbox.add(fake._msg('m30', 'Avis', thread: 't30'));
      fake.systemPoses.add({
        'thread': 't30',
        'kinds': ['mention'],
      });
      final src = await EngineMailSource.load(fake);
      final inbox = src.boxes['acc1']!.where((m) => m.folder == 'inbox');
      expect(inbox.where((m) => m.flags.contains('mention')).map((m) => m.id), [
        'acc1/inbox/t30',
      ]);
    },
  );

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

  test('un message sans partie texte n\'affiche pas « null »', () async {
    final fake = FakeEngine()..noText.add('m3');
    final src = await EngineMailSource.load(fake);
    final solo = src.boxes['acc1']!.firstWhere((m) => m.subject == 'Seul');
    await src.hydrate(solo);
    final shown = plainOf(solo.msgs.single.body);
    expect(shown, isNot(contains('null')));
    expect(shown, contains('Rouge'));
  });

  test(
    'un corps qui n\'a pas pu être chargé est redemandé à la réouverture',
    () async {
      final fake = FakeEngine()
        ..bodyFails.add('m3')
        ..fetchFails = true;
      final src = await EngineMailSource.load(fake);
      final solo = src.boxes['acc1']!.firstWhere((m) => m.subject == 'Seul');
      await src.hydrate(solo);
      fake.fetchFails = false;
      expect(await src.hydrate(solo), isTrue);
      expect(plainOf(solo.msgs.single.body), contains('Téléchargé'));
    },
  );

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

    Future<(MailStore, FakeEngine)> startWithDraft(WidgetTester tester) async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final fake = FakeEngine();
      fake.drafts['dr1'] = {
        'id': 'dr1',
        'account': 'acc1',
        'identity': 'id1',
        'to': [],
        'cc': [],
        'bcc': [],
        'subject': 'Brouillon à jeter',
        'body_text': 'texte',
        'body_html': null,
        'attachments': [],
        'in_reply_to': null,
        'state': {'type': 'editing'},
        'updated_at': 5000,
      };
      final src = await tester.runAsync(() => EngineMailSource.load(fake));
      final store = MailStore(prefs, source: src);
      expect(store.mails.where((m) => m.id == 'draft:dr1'), isNotEmpty);
      return (store, fake);
    }

    testWidgets('supprimer un brouillon de la liste le supprime du moteur', (
      tester,
    ) async {
      final (store, fake) = await startWithDraft(tester);
      store.selectFolder('drafts');
      store.sel.add('draft:dr1');
      store.trash();
      expect(store.mails.where((m) => m.id == 'draft:dr1'), isEmpty);
      await tester.pump(const Duration(seconds: 7));
      await tester.runAsync(() => Future<void>.delayed(Duration.zero));
      expect(fake.composed.last['op'], 'delete_draft');
      expect(fake.composed.last['draft'], 'dr1');
      expect(fake.drafts, isEmpty);
      store.dispose();
    });

    testWidgets('archiver ou signaler un brouillon le laisse en place', (
      tester,
    ) async {
      final (store, fake) = await startWithDraft(tester);
      store.selectFolder('drafts');
      store.sel.add('draft:dr1');
      store.archive();
      store.sel.add('draft:dr1');
      store.spam();
      expect(store.toasts.last.err, isTrue);
      await tester.pump(const Duration(seconds: 7));
      await tester.runAsync(() => Future<void>.delayed(Duration.zero));
      final draft = store.mails.singleWhere((m) => m.id == 'draft:dr1');
      expect(draft.folder, 'drafts');
      expect(fake.actions, isEmpty);
      store.dispose();
    });

    testWidgets('favori et lu/non lu ne s\'appliquent pas à un brouillon', (
      tester,
    ) async {
      final (store, fake) = await startWithDraft(tester);
      store.selectFolder('drafts');
      final draft = store.mails.singleWhere((m) => m.id == 'draft:dr1');
      final unread = draft.unread;
      store.toggleStar(draft.id);
      expect(draft.starred, isFalse);
      expect(store.toasts.last.err, isTrue);
      store.sel.add(draft.id);
      store.toggleRead();
      expect(draft.unread, unread);
      expect(store.toasts.last.err, isTrue);
      await tester.runAsync(() => Future<void>.delayed(Duration.zero));
      expect(fake.actions, isEmpty);
      store.dispose();
    });

    test(
      'retirer le favori depuis la vue Favoris passe par le moteur',
      () async {
        final (store, fake, conv) = await start();
        store.toggleStar(conv.id);
        await settle();
        store.selectFolder('starred');
        expect(store.visible(), contains(conv));
        store.toggleStar(conv.id);
        await settle();
        expect(fake.actions.last['kind'], {
          'type': 'mark_flagged',
          'flagged': false,
        });
        store.dispose();
      },
    );

    testWidgets('poser un libellé sur un brouillon ne change rien', (
      tester,
    ) async {
      final (store, fake) = await startWithDraft(tester);
      store.selectFolder('drafts');
      store.sel.add('draft:dr1');
      store.toggleLabel('mb-other');
      await tester.runAsync(() => Future<void>.delayed(Duration.zero));
      final draft = store.mails.singleWhere((m) => m.id == 'draft:dr1');
      expect(draft.labels, isEmpty);
      expect(fake.actions, isEmpty);
      expect(store.toasts.last.err, isTrue);
      store.dispose();
    });

    testWidgets('annuler la suppression garde le brouillon dans le moteur', (
      tester,
    ) async {
      final (store, fake) = await startWithDraft(tester);
      store.selectFolder('drafts');
      store.sel.add('draft:dr1');
      store.trash();
      store.toasts.last.cb!();
      await tester.pump(const Duration(seconds: 7));
      await tester.runAsync(() => Future<void>.delayed(Duration.zero));
      expect(fake.composed.where((c) => c['op'] == 'delete_draft'), isEmpty);
      expect(store.mails.where((m) => m.id == 'draft:dr1'), isNotEmpty);
      store.dispose();
    });

    testWidgets('vider les brouillons supprime aussi ceux du moteur', (
      tester,
    ) async {
      final (store, fake) = await startWithDraft(tester);
      store.selectFolder('drafts');
      store.emptyFolder();
      await tester.pump(const Duration(seconds: 7));
      await tester.runAsync(() => Future<void>.delayed(Duration.zero));
      expect(fake.actions.last['op'], 'empty_mailbox');
      expect(fake.composed.last['op'], 'delete_draft');
      expect(fake.drafts, isEmpty);
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

    test(
      'se désabonner par mailto envoie le message sans toucher la rédaction',
      () async {
        final (store, fake) = await start();
        final news = Mail(
          id: 'acc1/inbox/news',
          folder: 'inbox',
          from: const Person('News', 'news@x.org'),
          to: const [Person('Alice', 'alice@example.org')],
          subject: 'Lettre',
          date: 1,
          account: 'acc1',
          msgs: [
            Msg(
              from: const Person('News', 'news@x.org'),
              date: 1,
              body: const [],
              att: const [],
              unsub: const Unsub(mailto: 'leave@x.org?subject=stop'),
            ),
          ],
        );
        await store.unsubscribe(news);
        expect(fake.composed.map((c) => c['op']), [
          'create_draft',
          'update_draft',
          'send_draft',
        ]);
        final changes = last(fake, 'update_draft')['changes'] as Map;
        expect(changes['to'], [
          {'name': null, 'address': 'leave@x.org'},
        ]);
        expect(changes['subject'], 'stop');
        expect(store.c!.subject.text, 'Salut');
        expect(store.isUnsubscribed(news), isTrue);
        store.dispose();
      },
    );
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
