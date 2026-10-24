import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_mail_source.dart';
import 'package:mira/state/store.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'engine_source_test.dart' show FakeEngine;

/// Ouvre le magasin sur le faux moteur.
Future<MailStore> open(FakeEngine fake) async {
  SharedPreferences.setMockInitialValues({});
  final prefs = await SharedPreferences.getInstance();
  final store = MailStore(prefs, source: await EngineMailSource.load(fake));
  await store.localLabelsReady;
  return store;
}

/// Annonce un événement du moteur et laisse l'app réagir.
Future<void> emit(FakeEngine fake, Map<String, dynamic> kind) async {
  fake.feed.add({'kind': kind});
  await Future<void>.delayed(const Duration(milliseconds: 400));
}

/// Annonce le compteur de non-lus d'une boîte.
Map<String, dynamic> counters(String mailbox, int unread) => {
  'type': 'mailbox_counters_updated',
  'account': 'acc1',
  'mailbox': mailbox,
  'total': unread + 10,
  'unread': unread,
};

int calls(FakeEngine fake, String op) =>
    fake.calls.where((c) => c == op).length;

void main() {
  test(
    'un compteur annoncé met la barre latérale à jour sans relire le compte',
    () async {
      final fake = FakeEngine();
      final store = await open(fake);
      expect(store.navCount('inbox'), 4321);
      final reads = calls(fake, 'mailbox.list_mailboxes');
      await emit(fake, counters('mb-inbox', 9000));
      expect(store.navCount('inbox'), 9000);
      await emit(fake, counters('mb-inbox', 5000));
      expect(store.navCount('inbox'), 5000);
      expect(calls(fake, 'mailbox.list_mailboxes'), reads);
      store.dispose();
    },
  );

  test('le compteur d’une boîte inconnue est ignoré', () async {
    final fake = FakeEngine();
    final store = await open(fake);
    await emit(fake, counters('mb-inconnue', 9000));
    expect(store.navCount('inbox'), 4321);
    store.dispose();
  });

  test('une action refusée ou un envoi échoué relit le compte', () async {
    final fake = FakeEngine();
    final store = await open(fake);
    final before = calls(fake, 'message.list_messages');
    await emit(fake, {
      'type': 'action_failed',
      'account': 'acc1',
      'action': 'a1',
      'code': 'server.refused',
      'reason': 'refusé',
    });
    final afterAction = calls(fake, 'message.list_messages');
    expect(afterAction, greaterThan(before));
    await emit(fake, {
      'type': 'send_failed',
      'account': 'acc1',
      'action': 'a2',
      'code': 'network.down',
    });
    expect(calls(fake, 'message.list_messages'), greaterThan(afterAction));
    store.dispose();
  });

  test('une boîte créée ou resynchronisée relit le compte', () async {
    final fake = FakeEngine();
    final store = await open(fake);
    final before = calls(fake, 'mailbox.list_mailboxes');
    await emit(fake, {
      'type': 'mailbox_created',
      'account': 'acc1',
      'mailbox': 'mb-new',
    });
    expect(calls(fake, 'mailbox.list_mailboxes'), greaterThan(before));
    store.dispose();
  });

  test(
    'un changement de compte annoncé par le moteur relit la liste des comptes',
    () async {
      final fake = FakeEngine();
      final store = await open(fake);
      final before = calls(fake, 'account.list_accounts');
      await emit(fake, {'type': 'account_updated', 'account': 'acc1'});
      expect(calls(fake, 'account.list_accounts'), greaterThan(before));
      store.dispose();
    },
  );

  test('un événement sans effet déclaré ne relit rien', () async {
    final fake = FakeEngine();
    final store = await open(fake);
    final before = fake.calls.length;
    await emit(fake, {
      'type': 'body_available',
      'account': 'acc1',
      'message': 'm1',
    });
    await emit(fake, {'type': 'evenement_du_futur', 'account': 'acc1'});
    expect(fake.calls.length, before);
    store.dispose();
  });
}
