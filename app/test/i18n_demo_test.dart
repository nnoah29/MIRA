import 'package:flutter_test/flutter_test.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:mira/data/documents.dart';
import 'package:mira/data/models.dart';
import 'package:mira/data/seed.dart';
import 'package:mira/i18n/i18n.dart';
import 'package:mira/state/store.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Premier bloc d'un document fictif : son titre.
String firstTitle(String file) {
  final page = documentPages(Attachment(file, 'PDF', '1 Ko')).first;
  return (page.whereType<DocTitle>().first).title;
}

void main() {
  setUpAll(() async {
    for (final l in AppLang.values) {
      await initializeDateFormatting(l.dateLocale);
    }
  });

  tearDown(() => appLang = AppLang.fr);

  test('les messages fictifs suivent la langue active', () {
    expect(seed().first.subject, 'Ton plan de projet a été validé');
    appLang = AppLang.en;
    final mail = seed().first;
    expect(mail.subject, 'Your project plan has been approved');
    expect(mail.msgs.first.body.toString(), isNotEmpty);
    expect(mail.msgs.first.att.first.s, '184 KB');
  });

  test('les documents fictifs suivent la langue active', () {
    expect(firstTitle('plan-validation.pdf'), 'Validation du plan de projet');
    appLang = AppLang.en;
    expect(firstTitle('plan-validation.pdf'), 'Project plan approval');
    expect(firstTitle('facture-septembre.pdf'), 'Invoice');
  });

  test('changer de langue en démonstration retraduit les messages', () async {
    SharedPreferences.setMockInitialValues({});
    final s = MailStore(await SharedPreferences.getInstance());
    expect(
      s.mails.any((m) => m.subject == 'Ton plan de projet a été validé'),
      isTrue,
    );
    s.setLang(AppLang.en);
    expect(
      s.mails.any((m) => m.subject == 'Your project plan has been approved'),
      isTrue,
    );
    expect(
      s.mails.any((m) => m.subject == 'Ton plan de projet a été validé'),
      isFalse,
    );
    s.setLang(AppLang.fr);
    expect(
      s.mails.any((m) => m.subject == 'Ton plan de projet a été validé'),
      isTrue,
    );
    s.dispose();
  });
}
