import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/engine_mail_source.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/accounts_settings.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

Map<String, dynamic> _acc(
  String id,
  String name,
  String state, {
  int days = 90,
  String auth = 'password',
}) => {
  'id': id,
  'display_name': name,
  'address': '$id@ex.org',
  'provider': 'generic',
  'auth': auth,
  'servers': {
    'imap': {'host': 'imap.ex.org', 'port': 993, 'security': 'tls'},
    'smtp': {'host': 'smtp.ex.org', 'port': 587, 'security': 'start_tls'},
  },
  'sync': {'window_days': days},
  'state': {'type': state},
};

class AdminEngine implements EngineClient {
  final calls = <Map<String, dynamic>>[];
  final accounts = [
    _acc('a1', 'Alice', 'connected'),
    _acc('a2', 'Bob', 'auth_required'),
  ];
  final identities = <String, List<Map<String, dynamic>>>{
    'a1': [
      {
        'id': 'i1',
        'account': 'a1',
        'address': 'a1@ex.org',
        'display_name': 'Alice',
        'signature': 'Cordialement',
        'is_default': true,
      },
      {
        'id': 'i2',
        'account': 'a1',
        'address': 'alias@ex.org',
        'display_name': 'Alias',
        'signature': null,
        'is_default': false,
      },
    ],
    'a2': [],
  };

  Map<String, dynamic>? last(String op) {
    for (final c in calls.reversed) {
      final r = c['request'] as Map<String, dynamic>;
      if ('${c['domain']}.${r['op']}' == op) return r;
    }
    return null;
  }

  Map<String, dynamic> _find(String id) =>
      accounts.firstWhere((a) => a['id'] == id);

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async {
    calls.add(request);
    final r = request['request'] as Map<String, dynamic>;
    final op = '${request['domain']}.${r['op']}';
    switch (op) {
      case 'account.list_accounts':
        return {'type': 'accounts', 'data': accounts};
      case 'engine.get_status':
        return {
          'type': 'status',
          'data': {
            'accounts': [
              for (final a in accounts)
                {'account': a, 'pending_actions': a['id'] == 'a2' ? 3 : 0},
            ],
          },
        };
      case 'mailbox.list_mailboxes':
        return {'type': 'mailboxes', 'data': []};
      case 'account.update_account':
        final a = _find('${r['account']}');
        final ch = r['changes'] as Map<String, dynamic>;
        if (ch['display_name'] != null) a['display_name'] = ch['display_name'];
        if (ch['window_days'] != null) {
          (a['sync'] as Map)['window_days'] = ch['window_days'];
        }
        return {'type': 'account', 'data': a};
      case 'account.pause_account':
        (_find('${r['account']}')['state'] as Map)['type'] = 'paused';
        return {'type': 'account', 'data': _find('${r['account']}')};
      case 'account.resume_account':
        (_find('${r['account']}')['state'] as Map)['type'] = 'offline';
        return {'type': 'account', 'data': _find('${r['account']}')};
      case 'account.delete_account':
        accounts.removeWhere((a) => a['id'] == r['account']);
        return {'type': 'done'};
      case 'auth.store_app_password':
        (_find('${r['account']}')['state'] as Map)['type'] = 'connected';
        return {'type': 'done'};
      case 'identity.list_identities':
        return {
          'type': 'identities',
          'data': identities['${r['account']}'] ?? [],
        };
      case 'identity.add_identity':
        identities['${r['account']}']!.add({
          'id': 'i${identities.length + 10}',
          'account': r['account'],
          'address': r['address'],
          'display_name': r['display_name'],
          'signature': r['signature'],
          'is_default': r['is_default'],
        });
        return {'type': 'identity', 'data': {}};
      case 'identity.update_identity':
        for (final l in identities.values) {
          for (final i in l) {
            if (i['id'] == r['identity']) {
              i['display_name'] = r['display_name'];
              i['signature'] = r['signature'];
            }
          }
        }
        return {'type': 'identity', 'data': {}};
      case 'identity.set_default_identity':
        for (final i in identities['a1']!) {
          i['is_default'] = i['id'] == r['identity'];
        }
        return {'type': 'identity', 'data': {}};
    }
    throw EngineException('internal.bug', 'op inconnue $op');
  }

  @override
  Stream<Map<String, dynamic>> events({int? after}) => const Stream.empty();

  @override
  Future<String?> originalHtml(String messageId) async => null;

  @override
  Future<void> close() async {}
}

Future<(MailStore, AdminEngine)> _pump(WidgetTester tester) async {
  tester.view.physicalSize = const Size(1500, 3000);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  SharedPreferences.setMockInitialValues({});
  final engine = AdminEngine();
  final src = await EngineMailSource.load(engine);
  final store = MailStore(await SharedPreferences.getInstance(), source: src);
  await tester.pumpWidget(
    ChangeNotifierProvider.value(
      value: store,
      child: MaterialApp(
        theme: buildTheme(true),
        home: const Scaffold(
          body: SingleChildScrollView(child: AccountsSettings()),
        ),
      ),
    ),
  );
  for (var i = 0; i < 4; i++) {
    await tester.pump(const Duration(milliseconds: 20));
  }
  return (store, engine);
}

Finder _txt(String text) => find.textContaining(
  RegExp('^${RegExp.escape(text)}\$', caseSensitive: false),
);

Future<void> _tap(WidgetTester t, String key) async {
  await t.ensureVisible(find.byKey(ValueKey(key)));
  await t.tap(find.byKey(ValueKey(key)));
  for (var i = 0; i < 4; i++) {
    await t.pump(const Duration(milliseconds: 20));
  }
}

Future<void> _type(WidgetTester t, String key, String text) async {
  await t.ensureVisible(find.byKey(ValueKey(key)));
  await t.enterText(find.byKey(ValueKey(key)), text);
  await t.pump();
}

Future<void> _settle(WidgetTester t) async {
  await t.pump();
  await t.pump(const Duration(seconds: 5));
}

void main() {
  testWidgets('comptes avec état, actions en attente et reconnexion demandée', (
    tester,
  ) async {
    await _pump(tester);
    expect(find.text('Alice'), findsWidgets);
    expect(find.text('Bob'), findsWidgets);
    expect(_txt('Connecté'), findsOneWidget);
    expect(_txt('Reconnexion nécessaire'), findsOneWidget);
    expect(_txt('3 en attente'), findsOneWidget);
    await _tap(tester, 'account-head-a2');
    expect(find.text('Ce compte doit être reconnecté.'), findsOneWidget);
  });

  testWidgets('renommer, fenêtre de synchro, pause puis reprise', (
    tester,
  ) async {
    final (store, engine) = await _pump(tester);
    await _type(tester, 'name-a1', 'Alice Martin');
    await _tap(tester, 'save-name-a1');
    expect(
      (engine.last('account.update_account')!['changes']
          as Map)['display_name'],
      'Alice Martin',
    );
    expect(store.accounts.firstWhere((a) => a.id == 'a1').name, 'Alice Martin');
    await _tap(tester, 'window-a1-180');
    expect(
      (engine.last('account.update_account')!['changes'] as Map)['window_days'],
      180,
    );
    await _tap(tester, 'pause-a1');
    expect(engine.last('account.pause_account')!['account'], 'a1');
    expect(_txt('Reprendre'), findsOneWidget);
    await _tap(tester, 'pause-a1');
    expect(engine.last('account.resume_account')!['account'], 'a1');
    await _settle(tester);
  });

  testWidgets('suppression en deux temps : rien avant la confirmation', (
    tester,
  ) async {
    final (store, engine) = await _pump(tester);
    await _tap(tester, 'delete-a1');
    expect(engine.last('account.delete_account'), isNull);
    expect(_txt('Supprimer définitivement'), findsOneWidget);
    await _tap(tester, 'confirm-delete-a1');
    expect(engine.last('account.delete_account')!['account'], 'a1');
    expect(store.accounts.map((a) => a.id), ['a2']);
    expect(store.activeId, 'a2');
    await _settle(tester);
  });

  testWidgets('reconnexion par mot de passe : secret envoyé puis vidé', (
    tester,
  ) async {
    final (_, engine) = await _pump(tester);
    await _tap(tester, 'account-head-a2');
    await _type(tester, 'newpw-a2', 'nouveau-secret');
    await _tap(tester, 'storepw-a2');
    expect(engine.last('auth.store_app_password'), {
      'op': 'store_app_password',
      'account': 'a2',
      'password': 'nouveau-secret',
    });
    expect(find.text('nouveau-secret'), findsNothing);
    await _settle(tester);
  });

  testWidgets('serveurs : validation puis envoi', (tester) async {
    final (_, engine) = await _pump(tester);
    await _type(tester, 'imap-a1-port', '0');
    await _tap(tester, 'save-servers-a1');
    expect(find.byKey(const ValueKey('error-a1')), findsOneWidget);
    expect(
      engine.last('account.update_account')?['changes'] is Map &&
          (engine.last('account.update_account')!['changes'] as Map)
              .containsKey('servers'),
      isFalse,
    );
  });

  testWidgets('identités : liste, défaut, modification, ajout validé', (
    tester,
  ) async {
    final (_, engine) = await _pump(tester);
    expect(find.text('alias@ex.org'), findsOneWidget);
    expect(find.byKey(const ValueKey('default-i1')), findsNothing);
    expect(find.byKey(const ValueKey('default-i2')), findsOneWidget);
    await _tap(tester, 'default-i2');
    expect(engine.last('identity.set_default_identity')!['identity'], 'i2');
    await _type(tester, 'id-sig-i2', 'Bien à vous');
    await _tap(tester, 'save-identity-i2');
    expect(
      engine.last('identity.update_identity')!['signature'],
      'Bien à vous',
    );
    await _type(tester, 'new-id-address-a1', 'pas-une-adresse');
    await _tap(tester, 'add-identity-a1');
    expect(find.byKey(const ValueKey('identity-error-a1')), findsOneWidget);
    expect(engine.last('identity.add_identity'), isNull);
    await _type(tester, 'new-id-address-a1', 'nouveau@ex.org');
    await _type(tester, 'new-id-name-a1', 'Nouveau');
    await _tap(tester, 'new-id-default-a1-true');
    await _tap(tester, 'add-identity-a1');
    final add = engine.last('identity.add_identity')!;
    expect(add['address'], 'nouveau@ex.org');
    expect(add['display_name'], 'Nouveau');
    expect(add['is_default'], true);
    expect(find.text('nouveau@ex.org'), findsOneWidget);
  });
}
