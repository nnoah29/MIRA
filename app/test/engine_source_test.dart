import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/engine_mail_source.dart';
import 'package:mira/state/store.dart';
import 'package:shared_preferences/shared_preferences.dart';

class FakeEngine implements EngineClient {
  final calls = <String>[];
  final bodyFails = <String>{};

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
        return {
          'type': 'safe_html',
          'data': {
            'html': '<p>Bonjour <strong>Alice</strong></p>',
            'blocked_resources': [],
          },
        };
      case 'message.list_attachments':
        return {
          'type': 'attachments',
          'data': [
            {
              'filename': 'devis.pdf',
              'mime_type': 'application/pdf',
              'size': 2048,
            },
          ],
        };
    }
    throw EngineException('internal.bug', 'op inconnue $op');
  }

  Map<String, dynamic> _list(String mailbox) {
    final items = switch (mailbox) {
      'mb-inbox' => [
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
  Stream<Map<String, dynamic>> events({int? after}) => const Stream.empty();

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
    expect(conv.msgs[1].att.single.n, 'devis.pdf');
    expect(conv.msgs[1].att.single.t, 'PDF');
    expect(await src.hydrate(conv), isFalse);
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
}
