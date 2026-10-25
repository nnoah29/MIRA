import 'package:flutter/material.dart';

import '../engine/template_admin.dart';
import '../i18n/i18n.dart';
import '../state/template_variables.dart';
import '../theme/app_theme.dart';
import 'rule_widgets.dart';
import 'widgets.dart';

/// Éditeur d'un modèle : nom, objet, corps et barre de variables insérées au curseur.
class TemplateEditor extends StatefulWidget {
  final EngineTemplate? template;
  final String? error;
  final Future<void> Function(String name, String? subject, String body) onSave;
  final VoidCallback onCancel;
  const TemplateEditor({
    super.key,
    required this.template,
    required this.error,
    required this.onSave,
    required this.onCancel,
  });

  @override
  State<TemplateEditor> createState() => _TemplateEditorState();
}

class _TemplateEditorState extends State<TemplateEditor> {
  late final name = TextEditingController(text: widget.template?.name ?? '');
  late final subject = TextEditingController(
    text: widget.template?.subject ?? '',
  );
  late final body = TextEditingController(text: widget.template?.body ?? '');
  final bodyFocus = FocusNode();

  @override
  void dispose() {
    name.dispose();
    subject.dispose();
    body.dispose();
    bodyFocus.dispose();
    super.dispose();
  }

  /// Insère `{{variable}}` à la place de la sélection du corps, ou à la fin s'il n'y en a pas.
  void _insert(String variable) {
    final text = '{{$variable}}';
    final sel = body.selection;
    final start = sel.isValid ? sel.start : body.text.length;
    final end = sel.isValid ? sel.end : body.text.length;
    body.value = TextEditingValue(
      text: body.text.replaceRange(start, end, text),
      selection: TextSelection.collapsed(offset: start + text.length),
    );
    bodyFocus.requestFocus();
  }

  void _save() {
    final line = subject.text.trim();
    widget.onSave(name.text.trim(), line.isEmpty ? null : line, body.text);
  }

  Widget _field(
    Key key,
    TextEditingController controller,
    String hint, {
    int lines = 1,
    FocusNode? focus,
  }) {
    final p = context.pal;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      decoration: BoxDecoration(
        color: p.bg,
        border: Border.all(color: p.line),
      ),
      child: TextField(
        key: key,
        controller: controller,
        focusNode: focus,
        minLines: lines,
        maxLines: lines == 1 ? 1 : null,
        cursorColor: p.accent,
        cursorWidth: 1,
        style: Fonts.sans(size: 14, color: p.text),
        decoration: InputDecoration(
          isDense: true,
          border: InputBorder.none,
          contentPadding: EdgeInsets.zero,
          hintText: hint,
          hintStyle: Fonts.sans(size: 14, color: p.dim),
        ),
      ),
    );
  }

  Widget _variables() => Wrap(
    spacing: 6,
    runSpacing: 8,
    children: [
      for (final variable in templateVariables)
        RuleChip(
          key: ValueKey('template-var-$variable'),
          label: '{{$variable}}',
          selected: false,
          onTap: () => _insert(variable),
        ),
    ],
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Padding(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            widget.template == null
                ? t('Nouveau modèle')
                : t('Modifier le modèle'),
            style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
          ),
          const SizedBox(height: 14),
          _field(const ValueKey('template-name'), name, t('Nom du modèle')),
          const SizedBox(height: 10),
          _field(
            const ValueKey('template-subject'),
            subject,
            t('Objet (facultatif)'),
          ),
          const SizedBox(height: 10),
          _field(
            const ValueKey('template-body'),
            body,
            t('Texte du modèle'),
            lines: 8,
            focus: bodyFocus,
          ),
          const SizedBox(height: 12),
          Text(
            t(
              'Variables : touche-en une pour l’insérer à la place du curseur.',
            ),
            style: Fonts.sans(size: 13, color: p.dim),
          ),
          const SizedBox(height: 8),
          _variables(),
          if (widget.error != null)
            Padding(
              padding: const EdgeInsets.only(top: 12),
              child: Text(
                widget.error!,
                key: const ValueKey('template-error'),
                style: Fonts.sans(size: 13.5, color: p.accent),
              ),
            ),
          const SizedBox(height: 16),
          Row(
            children: [
              Btn(
                key: const ValueKey('save-template'),
                label: t('Enregistrer'),
                go: true,
                onTap: _save,
              ),
              const SizedBox(width: 10),
              Btn(
                key: const ValueKey('cancel-template'),
                label: t('Annuler'),
                onTap: widget.onCancel,
              ),
            ],
          ),
        ],
      ),
    );
  }
}
