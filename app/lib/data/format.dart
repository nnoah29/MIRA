import 'package:intl/intl.dart';

import '../i18n/i18n.dart';

const minMs = 60000;
const hourMs = 60 * minMs;
const dayMs = 24 * hourMs;

int nowMs() => DateTime.now().millisecondsSinceEpoch;

/// Instant d'envoi d'un choix de programmation (« dans 1 heure », « demain 08:00 », « lundi 08:00 »).
int scheduledMs(String choice, [DateTime? from]) {
  final now = from ?? DateTime.now();
  final c = choice.toLowerCase();
  if (c.startsWith('dans')) {
    return now.add(const Duration(hours: 1)).millisecondsSinceEpoch;
  }
  final days = c.startsWith('demain')
      ? 1
      : (DateTime.monday - now.weekday + 7) % 7 +
            (now.weekday == DateTime.monday ? 7 : 0);
  return DateTime(
    now.year,
    now.month,
    now.day + days,
    8,
  ).millisecondsSinceEpoch;
}

String pad2(int n) => n.toString().padLeft(2, '0');

String hhmm(int ms) {
  final d = DateTime.fromMillisecondsSinceEpoch(ms);
  return '${pad2(d.hour)}:${pad2(d.minute)}';
}

int _day0(int ms) {
  final d = DateTime.fromMillisecondsSinceEpoch(ms);
  return DateTime(d.year, d.month, d.day).millisecondsSinceEpoch;
}

int diffDays(int ms) => ((_day0(nowMs()) - _day0(ms)) / dayMs).round();

String rowTime(int ms) {
  final n = diffDays(ms);
  final d = DateTime.fromMillisecondsSinceEpoch(ms);
  if (n <= 0) return hhmm(ms);
  if (n == 1) return t('Hier');
  if (n < 7) return DateFormat('EEE', appLang.dateLocale).format(d);
  return DateFormat('d MMM', appLang.dateLocale).format(d);
}

String fullDate(int ms) => DateFormat(
  'EEE d MMM y, HH:mm',
  appLang.dateLocale,
).format(DateTime.fromMillisecondsSinceEpoch(ms));

String groupOf(int ms) {
  final n = diffDays(ms);
  return n <= 0
      ? "Aujourd'hui"
      : n == 1
      ? 'Hier'
      : n < 7
      ? 'Cette semaine'
      : 'Plus ancien';
}

String sizeOf(int b) => b < 1024
    ? '$b o'
    : b < 1048576
    ? '${(b / 1024).round()} Ko'
    : '${(b / 1048576).toStringAsFixed(1).replaceAll('.', ',')} Mo';

const avatarPalette = [
  0xFFd9b46a,
  0xFF6aa8d9,
  0xFFa79ae0,
  0xFF8bc48a,
  0xFFe08aa8,
  0xFFe8854b,
];

int colorOf(String s) {
  var h = 0;
  for (final c in s.runes) {
    // Même arithmétique que le JS d'origine (unités UTF-16, uint32).
    h = (h * 31 + c) & 0xFFFFFFFF;
  }
  return avatarPalette[h % avatarPalette.length];
}

String initials(String n) {
  final cleaned = n.replaceAll(RegExp(r'[^\p{L}\s]', unicode: true), '').trim();
  final w = cleaned.split(RegExp(r'\s+')).where((x) => x.isNotEmpty).toList();
  final s = w.length > 1
      ? w[0][0] + w[1][0]
      : (w.isEmpty ? '?' : (w[0].length >= 2 ? w[0].substring(0, 2) : w[0]));
  return s.toUpperCase();
}

String plural(int n, String w) => '$n $w${n > 1 ? 's' : ''}';

String stripRe(String s) =>
    s.replaceFirst(RegExp(r'^((re|fwd?)\s*:\s*)+', caseSensitive: false), '');
