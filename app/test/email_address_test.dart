import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/email_address.dart';

void main() {
  group('adresse acceptée par le moteur', () {
    test('formes usuelles', () {
      for (final a in [
        'bob@example.org',
        'a.b+c@sub.example.co.uk',
        "o'b@x.fr",
      ]) {
        expect(isEngineAddress(a), isTrue, reason: a);
      }
    });

    test('domaine terminé par un point, point en tête ou double point', () {
      for (final a in ['bob@example.org.', 'bob@.example.org', 'bob@a..org']) {
        expect(isEngineAddress(a), isFalse, reason: a);
      }
    });

    test('sans point dans le domaine, sans partie locale, deux @', () {
      for (final a in [
        'bob@localhost',
        '@example.org',
        'a@b@example.org',
        '',
      ]) {
        expect(isEngineAddress(a), isFalse, reason: a);
      }
    });

    test('espace dans l\'adresse', () {
      expect(isEngineAddress('bo b@example.org'), isFalse);
      expect(isEngineAddress('bob@exa mple.org'), isFalse);
    });
  });
}
