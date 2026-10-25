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

/// Rend le modèle dans le contexte de la rédaction et l'insère ; l'objet n'est rempli que s'il est vide.
Future<void> _insert(
  ComposeState c,
  TemplateAdmin admin,
  EngineTemplate chosen, {
  required String? fromName,
  required DateTime now,
  required bool fromSlash,
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
  insertAtCursor(c.body, rendered.body, removeSlash: fromSlash);
  final subject = rendered.subject;
  if (subject != null && c.subject.text.trim().isEmpty) {
    c.subject.text = subject;
  }
  touch();
  final left = rendered.unresolved.length;
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
  bool fromSlash = false,
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
      fromSlash: fromSlash,
      notify: notify,
      touch: touch,
    );
  } catch (e) {
    notify(engineErrorText(e), err: true);
  }
}
