import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/engine/sync_alert.dart';
import 'package:mira/engine/sync_state.dart';

const alice = Account('a1', 'Alice', 'alice@example.org');
const bob = Account('a2', 'Bob', 'bob@example.org');

/// État d'un compte dont la dernière synchro a échoué avec ce code.
SyncState failing(String code) =>
    SyncState()..apply('sync_failed', {'code': code});

/// État d'un compte hors ligne.
SyncState offline() => SyncState()..apply('went_offline', {});

void main() {
  test('chaque cause connue a son message, avec le compte', () {
    expect(
      syncAlertMessage(alice.email, failing('server.quota_exceeded')),
      contains('Boîte pleine'),
    );
    expect(
      syncAlertMessage(alice.email, failing('server.rate_limited')),
      contains('limite les requêtes'),
    );
    expect(
      syncAlertMessage(alice.email, failing('tls.invalid_certificate')),
      contains('Certificat'),
    );
    for (final code in ['server.quota_exceeded', 'tls.invalid_certificate']) {
      expect(
        syncAlertMessage(alice.email, failing(code)),
        contains(alice.email),
      );
    }
  });

  test('une cause inconnue garde le message générique avec son code', () {
    final message = syncAlertMessage(alice.email, failing('server.protocol'));
    expect(message, contains('impossible'));
    expect(message, contains('server.protocol'));
    expect(syncAlertMessage(alice.email, offline()), contains('Hors ligne'));
  });

  test('le compte le plus urgent passe en premier', () {
    (Account, SyncState) of(Account a, SyncState s) => (a, s);
    expect(
      mostUrgent([
        of(alice, offline()),
        of(bob, failing('server.rate_limited')),
      ])!.$1,
      bob,
    );
    expect(
      mostUrgent([
        of(alice, failing('server.rate_limited')),
        of(bob, failing('server.quota_exceeded')),
      ])!.$1,
      bob,
    );
    expect(
      mostUrgent([
        of(alice, failing('server.quota_exceeded')),
        of(bob, failing('tls.invalid_certificate')),
      ])!.$1,
      bob,
    );
  });

  test('sans problème, aucun bandeau d’alerte', () {
    final syncing = SyncState()..apply('sync_started', {});
    expect(mostUrgent([(alice, syncing)]), isNull);
    expect(mostUrgent(const []), isNull);
  });

  test('un échec disparaît à la synchro suivante réussie', () {
    final state = failing('server.quota_exceeded');
    state.apply('sync_finished', {'operation': 'op1'});
    expect(mostUrgent([(alice, state)]), isNull);
  });

  test('une alerte annoncée par le moteur (envoi, action) s’affiche et s’efface à l’envoi suivant', () {
    final state = SyncState();
    expect(state.apply('quota_full', {'account': 'a1'}), isTrue);
    expect(syncAlertMessage(alice.email, state), contains('Boîte pleine'));
    expect(mostUrgent([(alice, state)]), isNotNull);
    state.apply('send_departed', {'account': 'a1'});
    expect(mostUrgent([(alice, state)]), isNull);
    expect(state.apply('rate_limited', {'account': 'a1'}), isTrue);
    expect(
      syncAlertMessage(alice.email, state),
      contains('limite les requêtes'),
    );
  });

  test('un envoi réussi n’efface pas un autre échec de synchronisation', () {
    final state = failing('tls.invalid_certificate');
    state.apply('send_departed', {'account': 'a1'});
    expect(mostUrgent([(alice, state)]), isNotNull);
  });
}
