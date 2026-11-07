import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/notification_feed.dart';
import 'package:mira/state/mail_notifier.dart';

/// Afficheur de test : garde ce qu'on lui demande d'afficher.
class Display implements SystemNotifications {
  final shown = <(int, String, String, String?, bool)>[];
  Object? failure;

  @override
  Future<void> init(void Function(String payload) onTap) async {}

  @override
  Future<void> show(
    int id,
    String title,
    String body, {
    String? payload,
    bool sound = true,
  }) async {
    if (failure != null) throw failure!;
    shown.add((id, title, body, payload, sound));
  }
}

void main() {
  test('chaque notification du moteur est affichée telle quelle', () async {
    final stream = StreamController<Map<String, dynamic>>();
    final display = Display();
    final feed = NotificationFeed(stream.stream, display)..start();
    stream.add({
      'id': 7,
      'title': 'Bob',
      'body': 'Rendez-vous',
      'sound': false,
      'message': 'a1/m1',
    });
    await Future<void>.delayed(Duration.zero);
    expect(display.shown, [(7, 'Bob', 'Rendez-vous', 'a1/m1', false)]);
    await feed.stop();
  });

  test('un afficheur en panne n’arrête pas le flux', () async {
    final stream = StreamController<Map<String, dynamic>>();
    final display = Display()..failure = StateError('indisponible');
    final feed = NotificationFeed(stream.stream, display)..start();
    stream.add({'id': 1, 'title': 'a', 'body': 'b', 'sound': true});
    await Future<void>.delayed(Duration.zero);
    display.failure = null;
    stream.add({'id': 2, 'title': 'c', 'body': 'd', 'sound': true});
    await Future<void>.delayed(Duration.zero);
    expect(display.shown.single.$1, 2);
    await feed.stop();
  });

  test('après stop, plus rien n’est affiché', () async {
    final stream = StreamController<Map<String, dynamic>>();
    final display = Display();
    final feed = NotificationFeed(stream.stream, display)..start();
    await feed.stop();
    stream.add({'id': 3, 'title': 'x', 'body': 'y', 'sound': true});
    await Future<void>.delayed(Duration.zero);
    expect(display.shown, isEmpty);
  });
}
