import '../data/models.dart';

/// Nouveaux fils retenus jusqu'à ce que le moteur ait rendu, pour chacun de
/// leurs nouveaux messages, le verdict des règles (muet ou à annoncer).
class ArrivalGate {
  final Map<String, _Held> _held = {};

  /// Au-delà, un fil toujours sans verdict est abandonné sans notification.
  static const maxWaitMs = 10 * 60 * 1000;

  /// Retient chaque fil arrivé avec ses messages absents de `before`.
  void hold(List<Mail>? before, List<Mail> arrived, int nowMs) {
    final known = {
      for (final m in before ?? const <Mail>[])
        for (final x in m.msgs) ?x.ref,
    };
    for (final m in arrived) {
      final refs = {
        for (final x in m.msgs)
          if (x.ref != null && !known.contains(x.ref)) x.ref!,
      };
      if (refs.isEmpty) continue;
      final held = _held[m.id];
      _held[m.id] = _Held(m.account, {
        ...?held?.refs,
        ...refs,
      }, held?.since ?? nowMs);
    }
  }

  /// Fils du compte à annoncer : tous leurs nouveaux messages ont un verdict,
  /// au moins un n'est pas muet, et le fil est encore là et non lu.
  List<Mail> release(
    String account,
    List<Mail> fresh,
    bool? Function(String ref) verdict,
    void Function(Iterable<String> refs) forget,
    int nowMs,
  ) {
    final byId = {for (final m in fresh) m.id: m};
    final ready = <Mail>[];
    for (final e in [..._held.entries]) {
      final h = e.value;
      if (h.account != account) continue;
      final verdicts = [for (final r in h.refs) verdict(r)];
      if (verdicts.contains(null) && nowMs - h.since < maxWaitMs) continue;
      _held.remove(e.key);
      forget(h.refs);
      final current = byId[e.key];
      if (verdicts.contains(false) && current != null && current.unread) {
        ready.add(current);
      }
    }
    return ready;
  }
}

class _Held {
  final String account;
  final Set<String> refs;
  final int since;
  const _Held(this.account, this.refs, this.since);
}
