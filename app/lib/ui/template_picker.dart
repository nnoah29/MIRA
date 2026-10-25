import 'package:flutter/material.dart';

import '../engine/template_admin.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Sélecteur de modèle avec recherche par nom ; rend le modèle choisi, ou `null` si l'on ferme.
Future<EngineTemplate?> showTemplatePicker(
  BuildContext context,
  List<EngineTemplate> templates,
) => showDialog<EngineTemplate>(
  context: context,
  builder: (_) => _TemplatePicker(templates),
);

class _TemplatePicker extends StatefulWidget {
  final List<EngineTemplate> templates;
  const _TemplatePicker(this.templates);

  @override
  State<_TemplatePicker> createState() => _TemplatePickerState();
}

class _TemplatePickerState extends State<_TemplatePicker> {
  String query = '';

  List<EngineTemplate> get shown => [
    for (final item in widget.templates)
      if (item.name.toLowerCase().contains(query.trim().toLowerCase())) item,
  ];

  void _submit() {
    final found = shown;
    if (found.isNotEmpty) Navigator.pop(context, found.first);
  }

  Widget _row(EngineTemplate item) {
    final p = context.pal;
    return Hov(
      key: ValueKey('picker-row-${item.id}'),
      onTap: () => Navigator.pop(context, item),
      builder: (_, h) => Container(
        color: h ? p.surface : null,
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(item.name, style: Fonts.sans(size: 14.5, color: p.text)),
            if (item.subject != null)
              Text(
                item.subject!,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: Fonts.sans(size: 12.5, color: p.dim),
              ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final found = shown;
    return Dialog(
      backgroundColor: p.bgSide,
      shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 460, maxHeight: 460),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Padding(
              padding: const EdgeInsets.all(14),
              child: TextField(
                key: const ValueKey('template-search'),
                autofocus: true,
                textInputAction: TextInputAction.done,
                onChanged: (v) => setState(() => query = v),
                onSubmitted: (_) => _submit(),
                cursorColor: p.accent,
                style: Fonts.sans(size: 14.5, color: p.text),
                decoration: InputDecoration(
                  isDense: true,
                  border: InputBorder.none,
                  hintText: t('Chercher un modèle'),
                  hintStyle: Fonts.sans(size: 14.5, color: p.dim),
                ),
              ),
            ),
            Divider(height: 1, color: p.line),
            Flexible(
              child: found.isEmpty
                  ? Padding(
                      padding: const EdgeInsets.all(18),
                      child: Text(
                        widget.templates.isEmpty
                            ? t('Aucun modèle.')
                            : t('Aucun modèle trouvé.'),
                        style: Fonts.sans(size: 13.5, color: p.dim),
                      ),
                    )
                  : ListView(
                      shrinkWrap: true,
                      children: [for (final item in found) _row(item)],
                    ),
            ),
          ],
        ),
      ),
    );
  }
}
