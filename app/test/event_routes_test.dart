import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/event_routes.dart';

/// Types d'événements déclarés par le moteur : variantes de `EventKind`, en `snake_case`.
Set<String> engineEventTypes() {
  final source = File('../engine/crates/mira-vocabulary/src/events/kind.rs')
      .readAsStringSync();
  final start = source.indexOf('pub enum EventKind {');
  final body = source.substring(start, source.indexOf('\n}\n', start));
  final variant = RegExp(r'^    ([A-Z][A-Za-z]*)\b', multiLine: true);
  return {
    for (final m in variant.allMatches(body))
      m
          .group(1)!
          .replaceAllMapped(RegExp('(?<!^)[A-Z]'), (c) => '_${c.group(0)}')
          .toLowerCase(),
  };
}

void main() {
  test(
    'chaque événement du moteur a un traitement déclaré, et seulement eux',
    () {
      final declared = engineEventTypes();
      expect(declared.length, greaterThan(40));
      expect(
        declared.difference(eventRoutes.keys.toSet()),
        isEmpty,
        reason: 'événement du moteur sans traitement dans eventRoutes',
      );
      expect(
        eventRoutes.keys.toSet().difference(declared),
        isEmpty,
        reason: 'eventRoutes nomme un événement que le moteur ne connaît pas',
      );
    },
  );

  test('un événement qui change des données relit le compte ou met à jour un compteur', () {
    expect(effectsOf('message_updated'), {EventEffect.reload});
    expect(effectsOf('mailbox_counters_updated'), {EventEffect.counters});
    expect(effectsOf('new_message_received'), {
      EventEffect.reload,
      EventEffect.verdict,
    });
    expect(effectsOf('sync_finished'), {EventEffect.sync, EventEffect.reload});
    expect(effectsOf('action_failed'), {EventEffect.reload});
    expect(effectsOf('account_deleted'), {EventEffect.accounts});
    expect(effectsOf('quota_full'), {EventEffect.sync});
    expect(effectsOf('rate_limited'), {EventEffect.sync});
  });

  test('un type inconnu est ignoré sans erreur', () {
    expect(effectsOf('evenement_du_futur'), isEmpty);
  });
}
