import 'dart:convert';

/// Que faire des images distantes d'un message.
enum ImageAction {
  ask,
  always,
  never;

  static ImageAction parse(Object? v) =>
      values.firstWhere((a) => a.name == v, orElse: () => ask);
}

/// Règle pour une adresse (`a@b.com`) ou un domaine (`b.com`, sous-domaines compris).
class ImageRule {
  final String match;
  final ImageAction action;
  const ImageRule(this.match, this.action);

  Map<String, String> toJson() => {'match': match, 'action': action.name};
}

/// Normalise une saisie : adresse ou domaine en minuscules, sans `@` de tête ; `null` si invalide.
String? normalizeMatch(String input) {
  final s = input.trim().toLowerCase().replaceFirst(RegExp(r'^@'), '');
  final address = RegExp(r'^[^\s@]+@[^\s@]+\.[^\s@.]+$');
  final domain = RegExp(r'^([a-z0-9-]+\.)+[a-z]{2,}$');
  return address.hasMatch(s) || domain.hasMatch(s) ? s : null;
}

/// Politique d'affichage des images distantes : réglage général et règles par adresse ou domaine.
class ImagePolicy {
  final ImageAction fallback;
  final List<ImageRule> rules;
  const ImagePolicy({this.fallback = ImageAction.ask, this.rules = const []});

  /// Décision pour un expéditeur : adresse exacte, puis domaine le plus précis, puis réglage général.
  ImageAction decide(String email) {
    final address = email.trim().toLowerCase();
    for (final r in rules) {
      if (r.match == address) return r.action;
    }
    final domain = address.contains('@') ? address.split('@').last : address;
    ImageRule? best;
    for (final r in rules) {
      final hit =
          !r.match.contains('@') &&
          (domain == r.match || domain.endsWith('.${r.match}'));
      if (hit && (best == null || r.match.length > best.match.length)) best = r;
    }
    return best?.action ?? fallback;
  }

  /// Copie avec cette règle (remplace celle qui vise la même adresse ou le même domaine).
  ImagePolicy withRule(ImageRule rule) => ImagePolicy(
    fallback: fallback,
    rules: [...rules.where((r) => r.match != rule.match), rule],
  );

  ImagePolicy withoutRule(String match) => ImagePolicy(
    fallback: fallback,
    rules: rules.where((r) => r.match != match).toList(),
  );

  ImagePolicy withFallback(ImageAction a) =>
      ImagePolicy(fallback: a, rules: rules);

  String encode() => jsonEncode({
    'default': fallback.name,
    'rules': [for (final r in rules) r.toJson()],
  });

  /// Relit une politique enregistrée ; toute donnée illisible redonne « demander ».
  static ImagePolicy decode(String? raw) {
    if (raw == null) return const ImagePolicy();
    try {
      final j = jsonDecode(raw) as Map<String, dynamic>;
      return ImagePolicy(
        fallback: ImageAction.parse(j['default']),
        rules: [
          for (final r in (j['rules'] as List? ?? const []))
            if (normalizeMatch('${(r as Map)['match']}') case final m?)
              ImageRule(m, ImageAction.parse(r['action'])),
        ],
      );
    } catch (_) {
      return const ImagePolicy();
    }
  }
}
