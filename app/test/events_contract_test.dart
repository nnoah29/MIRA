import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/event_routes.dart';
import 'package:mira/engine/sync_state.dart';

/// Instantané des événements tel que le moteur les écrit sur le fil (vérifié côté Rust).
Map<String, dynamic> snapshot() => jsonDecode(
  File('../engine/crates/mira-core/tests/snapshots/events_v1.json')
      .readAsStringSync(),
) as Map<String, dynamic>;

/// Champs que l'app lit pour appliquer cet effet.
const fieldsRead = <EventEffect, List<String>>{
  EventEffect.reload: ['account'],
  EventEffect.sync: ['account'],
  EventEffect.counters: ['account', 'mailbox', 'unread'],
  EventEffect.verdict: ['message', 'silenced'],
};

void main() {
  test('chaque événement du fil porte les champs que son effet lit', () {
    final events = (snapshot()['events'] as Map).cast<String, dynamic>();
    for (final entry in events.entries) {
      final kind = (entry.value as Map).cast<String, dynamic>();
      for (final effect in effectsOf(entry.key)) {
        for (final field in fieldsRead[effect] ?? const <String>[]) {
          expect(
            kind.containsKey(field),
            isTrue,
            reason: '${entry.key} : champ « $field » absent pour $effect',
          );
        }
      }
    }
  });

  test('l’enveloppe a la forme que l’app lit : numéro, instant, nature', () {
    final envelope = (snapshot()['envelope'] as Map).cast<String, dynamic>();
    expect(envelope['seq'], isA<int>());
    expect(envelope['at'], isA<int>());
    final kind = (envelope['kind'] as Map).cast<String, dynamic>();
    expect(eventRoutes.containsKey(kind['type']), isTrue);
  });

  test('l’état de synchro lit les exemples réels sans erreur', () {
    final events = (snapshot()['events'] as Map).cast<String, dynamic>();
    final state = SyncState();
    for (final entry in events.entries) {
      state.apply(entry.key, (entry.value as Map).cast<String, dynamic>());
    }
    expect(state.failed, anyOf(isNull, isA<String>()));
  });

  test('le compteur annoncé a des nombres, pas des textes', () {
    final kind = (snapshot()['events']['mailbox_counters_updated'] as Map);
    expect(kind['unread'], isA<num>());
    expect(kind['total'], isA<num>());
  });
}
