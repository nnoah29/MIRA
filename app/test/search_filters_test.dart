import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/state/search_filters.dart';

const _day = Duration.millisecondsPerDay;

Mail _mail({
  bool unread = false,
  bool starred = false,
  String from = 'alice@example.org',
  int date = 0,
  String account = 'a1',
  List<String> labels = const [],
}) => Mail(
  id: 'm',
  folder: 'inbox',
  from: Person(from.split('@').first, from),
  to: const [],
  subject: 's',
  date: date,
  unread: unread,
  starred: starred,
  labels: [...labels],
  account: account,
  msgs: [],
);

void main() {
  const now = 100 * _day;

  test('sans filtre : vide, et tout passe', () {
    const f = SearchFilters();
    expect(f.isEmpty, isTrue);
    expect(f.engineActive, isFalse);
    expect(f.matches(_mail(), now), isTrue);
  });

  test('non lus, favoris, expéditeur et période se combinent', () {
    final f = const SearchFilters().copyWith(
      unread: true,
      from: 'ALICE',
      period: SearchPeriod.week,
    );
    expect(f.count, 3);
    expect(f.matches(_mail(unread: true, date: now - 2 * _day), now), isTrue);
    expect(f.matches(_mail(unread: false, date: now - _day), now), isFalse);
    expect(f.matches(_mail(unread: true, date: now - 9 * _day), now), isFalse);
    expect(
      f.matches(_mail(unread: true, from: 'bob@x.org', date: now), now),
      isFalse,
    );
  });

  test('compte et libellé sont filtrés par l\'application', () {
    final f = const SearchFilters().copyWith(account: 'a2', label: 'local:0');
    expect(f.engineActive, isFalse);
    expect(f.matches(_mail(account: 'a2', labels: ['local:0']), now), isTrue);
    expect(f.matches(_mail(account: 'a1', labels: ['local:0']), now), isFalse);
    expect(f.matches(_mail(account: 'a2'), now), isFalse);
  });

  test('requête du moteur : filtres communs et borne de période', () {
    final f = const SearchFilters().copyWith(
      attachments: true,
      from: ' bob ',
      period: SearchPeriod.day,
    );
    expect(f.engineCommon(now), {
      'unread_only': false,
      'flagged_only': false,
      'has_attachments': true,
      'from_contains': 'bob',
      'since': now - _day,
      'until': null,
    });
  });

  test('copyWith peut retirer le compte', () {
    final f = const SearchFilters(account: 'a1').copyWith(account: null);
    expect(f.account, isNull);
    expect(f.isEmpty, isTrue);
  });
}
