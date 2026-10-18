import 'package:flutter/material.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/state/contacts.dart';
import 'package:mira/state/store.dart';
import 'package:provider/provider.dart';
import 'package:mira/ui/contacts_settings.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:shared_preferences/shared_preferences.dart';

Mail _sent(int date, List<Person> to) => Mail(
  id: 's$date',
  folder: 'sent',
  from: const Person('Moi', 'moi@x.org'),
  to: to,
  subject: 'Objet',
  date: date,
  msgs: [],
);

void main() {
  setUpAll(() => initializeDateFormatting('fr_FR'));
  test('les destinataires des messages envoyés sont comptés et datés', () {
    final book = ContactBook({}, {});
    final changed = book.remember([
      _sent(100, const [Person('Bob', 'bob@x.org')]),
      _sent(300, const [
        Person('Bob', 'Bob@x.org'),
        Person('Eve', 'eve@x.org'),
      ]),
    ], (e) => e == 'moi@x.org');
    expect(changed, isTrue);
    final list = book.entries();
    expect(list.map((e) => e.email), ['bob@x.org', 'eve@x.org']);
    expect(list.first.count, 2);
    expect(list.first.last, 300);
  });

  test('l\'historique garde les envois que la boîte ne montre plus', () {
    final book = ContactBook({}, {
      'bob@x.org': ['Bob', 5, 900],
    });
    book.remember([
      _sent(100, const [Person('Bob', 'bob@x.org')]),
    ], (_) => false);
    expect(book.entries().single.count, 5);
    expect(book.entries().single.last, 900);
  });

  test('un nom choisi remplace le nom vu, et survit à l\'enregistrement', () {
    final book = ContactBook({'bob@x.org': 'Bobby'}, {});
    book.remember([
      _sent(1, const [Person('Bob', 'bob@x.org')]),
    ], (_) => false);
    expect(book.entries().single.name, 'Bobby');
    final back = ContactBook.decode(book.encodeNames(), book.encodeHistory());
    expect(back.entries().single.name, 'Bobby');
    expect(ContactBook.decode('pas du json', null).entries(), isEmpty);
  });

  test('le nom choisi s\'affiche à la place du nom reçu, dans la liste comme dans le lecteur', () async {
    SharedPreferences.setMockInitialValues({});
    final store = MailStore(await SharedPreferences.getInstance());
    const bob = Person('bob.x', 'Bob@X.org');
    expect(store.shown(bob), 'bob.x');
    store.setContactName('bob@x.org', 'Bob Martin');
    expect(store.shown(bob), 'Bob Martin');
    store.setContactName('bob@x.org', '');
    expect(store.shown(bob), 'bob.x');
    store.dispose();
  });

  testWidgets('l\'onglet Contacts permet de nommer une adresse', (
    tester,
  ) async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();
    final store = MailStore(prefs);
    store.mails.add(
      _sent(nowForTest, const [Person('bob@x.org', 'bob@x.org')]),
    );
    await tester.pumpWidget(
      ChangeNotifierProvider.value(
        value: store,
        child: MaterialApp(
          theme: buildTheme(true),
          home: const Scaffold(
            body: SingleChildScrollView(child: ContactsSettings()),
          ),
        ),
      ),
    );
    expect(
      find.byKey(const ValueKey('contact-name-bob@x.org')),
      findsOneWidget,
    );
    await tester.enterText(
      find.byKey(const ValueKey('contact-name-bob@x.org')),
      'Bobby',
    );
    await tester.pump();
    await tester.tap(find.byKey(const ValueKey('contact-save-bob@x.org')));
    await tester.pump();
    expect(store.contactBook.names['bob@x.org'], 'Bobby');
    expect(
      store.contactPeople().firstWhere((c) => c.email == 'bob@x.org').name,
      'Bobby',
    );
    await tester.pump(const Duration(seconds: 10));
    store.dispose();
  });
}

const nowForTest = 1700000000000;
