import 'package:flutter/widgets.dart';

import '../engine/template_admin.dart';
import '../i18n/engine_errors.dart';
import '../i18n/i18n.dart';
import '../state/store.dart';
import '../state/template_context.dart';
import '../state/template_insert.dart';
import 'template_picker.dart';

/// Annonce un message à l'utilisateur ; [err] le signale comme une erreur.
typedef Notify = void Function(String message, {bool err});

/// Charge les modèles puis laisse choisir ; `null` si l'on ferme le sélecteur.
Future<EngineTemplate?> _choose(
  BuildContext context,
  TemplateAdmin admin,
) async {
  final all = await admin.list();
  if (!context.mounted) return null;
  return showTemplatePicker(context, all);
}

/// Nombre de variables restées visibles dans les textes réellement insérés.
int _leftIn(List<String> unresolved, Iterable<String> inserted) {
  final shown = <String>{
    for (final text in inserted)
      for (final m in RegExp(r'\{\{\s*([^}]+?)\s*\}\}').allMatches(text))
        m.group(1)!,
  };
  return unresolved.where(shown.contains).length;
}

/// Rend le modèle dans le contexte de la rédaction et l'insère ; l'objet n'est rempli que s'il est vide.
Future<void> _insert(
  ComposeState c,
  TemplateAdmin admin,
  EngineTemplate chosen, {
  required String? fromName,
  required DateTime now,
  required int? slashAt,
  required Notify notify,
  required VoidCallback touch,
}) async {
  final context = buildTemplateContext(
    to: c.to,
    subject: c.subject.text,
    fromEmail: c.from,
    fromName: fromName,
    now: now,
  );
  final rendered = await admin.render(chosen.id, context);
  insertAtCursor(c.body, rendered.body, slashAt: slashAt);
  final subject = rendered.subject;
  final applied = subject != null && c.subject.text.trim().isEmpty
      ? subject
      : null;
  if (applied != null) c.subject.text = applied;
  touch();
  final left = _leftIn(rendered.unresolved, [rendered.body, ?applied]);
  if (left > 0) {
    notify(tn(left, 'variable à compléter', 'variables à compléter'));
  }
}

/// Ouvre le sélecteur de modèles puis insère le modèle choisi ; une erreur du moteur est annoncée, rien n'est modifié.
Future<void> pickAndInsertTemplate(
  BuildContext context, {
  required ComposeState c,
  required TemplateAdmin admin,
  required String? fromName,
  required Notify notify,
  required VoidCallback touch,
  DateTime? now,
  int? slashAt,
}) async {
  try {
    final chosen = await _choose(context, admin);
    if (chosen == null) return;
    await _insert(
      c,
      admin,
      chosen,
      fromName: fromName,
      now: now ?? DateTime.now(),
      slashAt: slashAt,
      notify: notify,
      touch: touch,
    );
  } catch (e) {
    notify(engineErrorText(e), err: true);
  }
}
