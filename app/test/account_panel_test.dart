import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/engine_mail_source.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/add_account_panel.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

Map<String, dynamic> _provider(
  String id,
  String name,
  List<String> domains,
  List<String> methods,
) => {
  'id': id,
  'display_name': name,
  'domains': domains,
  'auth_methods': [
    for (final m in methods) {'type': m},
  ],
};

class AccountsEngine implements EngineClient {
  final calls = <Map<String, dynamic>>[];
  final accounts = <Map<String, dynamic>>[];
  Object? failWith;

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async {
    calls.add(request);
    final r = request['request'] as Map<String, dynamic>;
    final op = '${request['domain']}.${r['op']}';
    final failure = failWith;
    if (failure != null &&
        (op == 'account.add_account' || op == 'account.test_connection')) {
      throw failure;
    }
    switch (op) {
      case 'provider.list_providers':
        return {
          'type': 'providers',
          'data': [
            _provider(
              'gmail',
              'Gmail',
              ['gmail.com'],
              ['oauth2', 'app_password'],
            ),
            _provider('yahoo', 'Yahoo', ['yahoo.com'], ['app_password']),
            _provider('generic', 'Autre', [], ['password', 'app_password']),
          ],
        };
      case 'account.prepare_account':
        final generic = r['provider'] == 'generic';
        return {
          'type': 'account_preparation',
          'data': {
            'provider': generic
                ? _provider('generic', 'Autre', [], ['password'])
                : _provider(
                    'gmail',
                    'Gmail',
                    ['gmail.com'],
                    ['oauth2', 'app_password'],
                  ),
            'servers': {
              'imap': {'host': 'imap.test', 'port': 993, 'security': 'tls'},
              'smtp': {
                'host': 'smtp.test',
                'port': 587,
                'security': 'start_tls',
              },
            },
          },
        };
      case 'account.test_connection':
        return {'type': 'done'};
      case 'account.add_account':
        accounts.add({
          'id': 'new1',
          'display_name': (r['draft'] as Map)['display_name'],
          'address': (r['draft'] as Map)['address'],
        });
        return {'type': 'account', 'data': accounts.last};
      case 'account.list_accounts':
        return {'type': 'accounts', 'data': accounts};
      case 'auth.start_oauth':
        return {
          'type': 'oauth_started',
          'data': {'flow': 'f1', 'authorize_url': 'file:///nope'},
        };
      case 'auth.complete_oauth':
      case 'auth.cancel_oauth':
      case 'sync.start_sync':
        return {'type': 'done'};
      case 'mailbox.list_mailboxes':
        return {'type': 'mailboxes', 'data': []};
    }
    throw EngineException('internal.bug', 'op inconnue $op');
  }

  Map<String, dynamic>? last(String op) {
    for (final c in calls.reversed) {
      final r = c['request'] as Map<String, dynamic>;
      if ('${c['domain']}.${r['op']}' == op) return r;
    }
    return null;
  }

  @override
  Stream<Map<String, dynamic>> events({int? after}) => const Stream.empty();

  @override
  Future<String?> originalHtml(String messageId) async => null;

  @override
  Future<void> close() async {}
}

Future<(MailStore, AccountsEngine)> _pump(WidgetTester tester) async {
  tester.view.physicalSize = const Size(1500, 1400);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  SharedPreferences.setMockInitialValues({});
  final engine = AccountsEngine();
  final src = await EngineMailSource.load(engine);
  final store = MailStore(await SharedPreferences.getInstance(), source: src);
  await tester.pumpWidget(
    ChangeNotifierProvider.value(
      value: store,
      child: MaterialApp(
        theme: buildTheme(true),
        home: const Scaffold(body: AddAccountPanel()),
      ),
    ),
  );
  await tester.pump();
  await tester.pump();
  return (store, engine);
}

Future<void> _settle(WidgetTester t) async {
  await t.pump();
  await t.pump(const Duration(seconds: 5));
}

Future<void> _type(WidgetTester t, String key, String text) async {
  await t.enterText(find.byKey(ValueKey(key)), text);
  await t.pump();
}

void main() {
  testWidgets('les fournisseurs du moteur sont des boutons', (tester) async {
    await _pump(tester);
    expect(find.byKey(const ValueKey('provider-gmail')), findsOneWidget);
    expect(find.byKey(const ValueKey('provider-yahoo')), findsOneWidget);
    expect(find.byKey(const ValueKey('provider-generic')), findsOneWidget);
    expect(find.text('Autre (IMAP / SMTP)'), findsOneWidget);
  });

  testWidgets('mot de passe d’application : ajout, secret vidé, compte actif', (
    tester,
  ) async {
    final (store, engine) = await _pump(tester);
    await tester.tap(find.byKey(const ValueKey('provider-yahoo')));
    await tester.pump();
    await _type(tester, 'field-name', 'Alice');
    await _type(tester, 'field-address', 'alice@yahoo.com');
    await _type(tester, 'field-password', 's3cret-pw');
    await tester.tap(find.byKey(const ValueKey('btn-add')));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 50));
    final add = engine.last('account.add_account')!;
    expect((add['draft'] as Map)['provider'], 'yahoo');
    expect((add['draft'] as Map)['auth'], 'app_password');
    expect((add['draft'] as Map)['servers'], isNull);
    expect(add['credential'], {'type': 'app_password', 'value': 's3cret-pw'});
    expect(store.activeId, 'new1');
    expect(engine.last('sync.start_sync'), isNotNull);
    expect(find.text('s3cret-pw'), findsNothing);
    await _settle(tester);
  });

  testWidgets('Autre : serveurs IMAP et SMTP validés puis envoyés', (
    tester,
  ) async {
    final (_, engine) = await _pump(tester);
    await tester.tap(find.byKey(const ValueKey('provider-generic')));
    await tester.pump();
    await _type(tester, 'field-address', 'bob@perso.fr');
    await tester.pump(const Duration(milliseconds: 600));
    await _type(tester, 'field-password', 'pw');
    await _type(tester, 'imap-port', '99999');
    await tester.tap(find.byKey(const ValueKey('btn-add')));
    await tester.pump();
    expect(find.textContaining('port invalide'), findsOneWidget);
    expect(engine.last('account.add_account'), isNull);
    await _type(tester, 'imap-host', 'mail.perso.fr');
    await _type(tester, 'imap-port', '143');
    await tester.tap(find.byKey(const ValueKey('imap-sec-start_tls')));
    await tester.pump();
    await _type(tester, 'smtp-host', 'smtp.perso.fr');
    await _type(tester, 'smtp-port', '465');
    await tester.tap(find.byKey(const ValueKey('smtp-sec-tls')));
    await tester.pump();
    await tester.tap(find.byKey(const ValueKey('btn-add')));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 50));
    final draft = engine.last('account.add_account')!['draft'] as Map;
    await _settle(tester);
    expect(draft['provider'], 'generic');
    expect(draft['servers'], {
      'imap': {'host': 'mail.perso.fr', 'port': 143, 'security': 'start_tls'},
      'smtp': {'host': 'smtp.perso.fr', 'port': 465, 'security': 'tls'},
    });
  });

  testWidgets(
    'tester la connexion n’ajoute rien ; une erreur du moteur s’affiche sans secret',
    (tester) async {
      final (_, engine) = await _pump(tester);
      await tester.tap(find.byKey(const ValueKey('provider-yahoo')));
      await tester.pump();
      await _type(tester, 'field-address', 'alice@yahoo.com');
      await _type(tester, 'field-password', 'topsecret');
      await tester.tap(find.byKey(const ValueKey('btn-test')));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 50));
      expect(find.byKey(const ValueKey('form-info')), findsOneWidget);
      expect(engine.last('account.add_account'), isNull);
      engine.failWith = const EngineException(
        'authentication.rejected',
        'Identifiants refusés par le serveur',
      );
      await tester.tap(find.byKey(const ValueKey('btn-test')));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 50));
      expect(find.text('Identifiants refusés par le serveur'), findsOneWidget);
      expect(
        find.descendant(
          of: find.byKey(const ValueKey('form-error')),
          matching: find.textContaining('topsecret'),
        ),
        findsNothing,
      );
    },
  );

  testWidgets(
    'OAuth : flux démarré, terminé, puis compte ajouté sans mot de passe',
    (tester) async {
      final (store, engine) = await _pump(tester);
      await tester.tap(find.byKey(const ValueKey('provider-gmail')));
      await tester.pump();
      await _type(tester, 'field-address', 'me@gmail.com');
      await tester.tap(find.byKey(const ValueKey('method-oauth2')));
      await tester.pump();
      await tester.tap(find.byKey(const ValueKey('btn-oauth')));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 100));
      expect(engine.last('auth.start_oauth'), {
        'op': 'start_oauth',
        'address': 'me@gmail.com',
        'provider': 'gmail',
      });
      expect(engine.last('auth.complete_oauth'), isNotNull);
      final add = engine.last('account.add_account')!;
      expect(add['credential'], {
        'type': 'oauth_flow',
        'value': {'flow': 'f1'},
      });
      expect(store.activeId, 'new1');
      await _settle(tester);
    },
  );

  testWidgets('adresse inconnue : message clair, rien n’est envoyé', (
    tester,
  ) async {
    final (_, engine) = await _pump(tester);
    await tester.tap(find.byKey(const ValueKey('provider-yahoo')));
    await tester.pump();
    await _type(tester, 'field-address', 'pas-une-adresse');
    await tester.tap(find.byKey(const ValueKey('btn-add')));
    await tester.pump();
    expect(find.text('Adresse e-mail invalide.'), findsOneWidget);
    expect(engine.last('account.add_account'), isNull);
  });
}
