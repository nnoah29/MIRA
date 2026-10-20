import '../data/models.dart';

/// Période proposée pour restreindre une recherche.
enum SearchPeriod {
  any(0),
  day(1),
  week(7),
  month(30);

  final int days;
  const SearchPeriod(this.days);
}

/// Filtres d'une recherche, combinés par « et » avec le texte tapé.
class SearchFilters {
  final bool unread, starred, attachments;
  final String from;
  final SearchPeriod period;

  /// Compte et libellé : appliqués par l'application (le moteur ne les filtre pas).
  final String? account, label;

  const SearchFilters({
    this.unread = false,
    this.starred = false,
    this.attachments = false,
    this.from = '',
    this.period = SearchPeriod.any,
    this.account,
    this.label,
  });

  bool get isEmpty => count == 0;

  /// Nombre de filtres posés.
  int get count =>
      (unread ? 1 : 0) +
      (starred ? 1 : 0) +
      (attachments ? 1 : 0) +
      (from.trim().isEmpty ? 0 : 1) +
      (period == SearchPeriod.any ? 0 : 1) +
      (account == null ? 0 : 1) +
      (label == null ? 0 : 1);

  /// Les filtres que le moteur sait appliquer lui-même, dans le format de sa requête.
  Map<String, Object?> engineCommon(int now) => {
    'unread_only': unread,
    'flagged_only': starred,
    'has_attachments': attachments,
    'from_contains': from.trim().isEmpty ? null : from.trim(),
    'since': period == SearchPeriod.any
        ? null
        : now - period.days * Duration.millisecondsPerDay,
    'until': null,
  };

  /// Le moteur peut-il, à lui seul, répondre à une recherche sans texte ? (au moins un de ses filtres posé)
  bool get engineActive =>
      unread ||
      starred ||
      attachments ||
      from.trim().isNotEmpty ||
      period != SearchPeriod.any;

  /// Une conversation passe-t-elle tous les filtres ?
  bool matches(Mail m, int now) {
    if (unread && !m.unread) return false;
    if (starred && !m.starred) return false;
    if (attachments &&
        !m.msgs.any((x) => x.hasAttachments || x.att.isNotEmpty)) {
      return false;
    }
    final needle = from.trim().toLowerCase();
    if (needle.isNotEmpty &&
        !'${m.from.name} ${m.from.email}'.toLowerCase().contains(needle)) {
      return false;
    }
    if (period != SearchPeriod.any &&
        m.date < now - period.days * Duration.millisecondsPerDay) {
      return false;
    }
    if (account != null && m.account != account) return false;
    return label == null || m.labels.contains(label);
  }

  SearchFilters copyWith({
    bool? unread,
    bool? starred,
    bool? attachments,
    String? from,
    SearchPeriod? period,
    Object? account = _keep,
    Object? label = _keep,
  }) => SearchFilters(
    unread: unread ?? this.unread,
    starred: starred ?? this.starred,
    attachments: attachments ?? this.attachments,
    from: from ?? this.from,
    period: period ?? this.period,
    account: identical(account, _keep) ? this.account : account as String?,
    label: identical(label, _keep) ? this.label : label as String?,
  );

  @override
  bool operator ==(Object other) =>
      other is SearchFilters &&
      other.unread == unread &&
      other.starred == starred &&
      other.attachments == attachments &&
      other.from == from &&
      other.period == period &&
      other.account == account &&
      other.label == label;

  @override
  int get hashCode =>
      Object.hash(unread, starred, attachments, from, period, account, label);
}

const _keep = Object();
