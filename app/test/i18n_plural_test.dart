import 'package:flutter_test/flutter_test.dart';
import 'package:mira/i18n/i18n.dart';

void main() {
  tearDown(() => appLang = AppLang.fr);

  test('français : 0 et 1 au singulier, le reste au pluriel', () {
    expect(
      [
        for (final n in [0, 1, 2, 5]) pluralFr(n),
      ],
      ['one', 'one', 'other', 'other'],
    );
    expect(tn(0, 'message', 'messages'), '0 message');
    expect(tn(1, 'message', 'messages'), '1 message');
    expect(tn(3, 'message', 'messages'), '3 messages');
  });

  test('anglais : seul 1 est au singulier', () {
    appLang = AppLang.en;
    expect(
      [
        for (final n in [0, 1, 2]) pluralEn(n),
      ],
      ['other', 'one', 'other'],
    );
    expect(tn(0, 'message', 'messages'), '0 messages');
    expect(tn(1, 'message', 'messages'), '1 message');
    expect(tn(2, 'message', 'messages'), '2 messages');
  });

  test(
    'une langue à plusieurs formes écrit les siennes, avec repli sur « autre »',
    () {
      String translate(String s) => 'tr-$s';
      String? lookup(String key) => const {'messages#few': 'few-form'}[key];
      expect(
        pluralText('one', 'message', 'messages', translate, lookup),
        'tr-message',
      );
      expect(
        pluralText('other', 'message', 'messages', translate, lookup),
        'tr-messages',
      );
      expect(
        pluralText('few', 'message', 'messages', translate, lookup),
        'few-form',
      );
      expect(
        pluralText('many', 'message', 'messages', translate, lookup),
        'tr-messages',
      );
    },
  );

  test('chaque langue donne une catégorie pour tout nombre', () {
    for (final lang in AppLang.values) {
      for (final n in [0, 1, 2, 3, 5, 11, 21, 100]) {
        expect(lang.plural(n), isNotEmpty, reason: '${lang.code} $n');
      }
    }
  });
}
