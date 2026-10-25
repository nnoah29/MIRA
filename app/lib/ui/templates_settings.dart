import 'dart:async';

import 'package:flutter/material.dart';

import '../engine/template_admin.dart';
import '../i18n/engine_errors.dart';
import '../i18n/i18n.dart';
import '../state/templates.dart';
import '../theme/app_theme.dart';
import 'template_editor.dart';
import 'widgets.dart';

/// Modèles de messages : liste, création, modification et suppression.
class TemplatesSettings extends StatefulWidget {
  final TemplateAdmin admin;
  final Stream<void> changes;
  const TemplatesSettings({
    super.key,
    required this.admin,
    required this.changes,
  });

  @override
  State<TemplatesSettings> createState() => _TemplatesSettingsState();
}

class _TemplatesSettingsState extends State<TemplatesSettings> {
  late final Templates state = Templates(widget.admin, widget.changes);
  EngineTemplate? editing;
  bool creating = false;
  String? error;

  @override
  void initState() {
    super.initState();
    state.addListener(_refresh);
    state.reload();
  }

  void _refresh() {
    if (mounted) setState(() {});
  }

  @override
  void dispose() {
    state.removeListener(_refresh);
    state.dispose();
    super.dispose();
  }

  Future<void> _save(String name, String? subject, String body) async {
    try {
      final current = editing;
      if (current == null) {
        await widget.admin.create(name, subject, body);
      } else {
        await widget.admin.update(current.id, name, subject, body);
      }
      if (!mounted) return;
      setState(() {
        editing = null;
        creating = false;
        error = null;
      });
    } catch (e) {
      if (mounted) setState(() => error = engineErrorText(e));
    }
    await state.reload();
  }

  Future<void> _delete(EngineTemplate template) async {
    try {
      await widget.admin.delete(template.id);
      error = null;
    } catch (e) {
      error = engineErrorText(e);
    }
    await state.reload();
  }

  Future<void> _confirmDelete(EngineTemplate template) async {
    final p = context.pal;
    final ok = await showDialog<bool>(
      context: context,
      builder: (c) => Dialog(
        backgroundColor: p.bgSide,
        shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 420),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  t('Supprimer le modèle « {0} » ?', [template.name]),
                  style: Fonts.sans(
                    size: 16,
                    w: FontWeight.w600,
                    color: p.text,
                  ),
                ),
                const SizedBox(height: 20),
                Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    Btn(
                      label: t('Annuler'),
                      onTap: () => Navigator.pop(c, false),
                    ),
                    const SizedBox(width: 10),
                    Btn(
                      key: const ValueKey('confirm-delete-template'),
                      label: t('Supprimer'),
                      go: true,
                      onTap: () => Navigator.pop(c, true),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      ),
    );
    if (ok == true) await _delete(template);
  }

  /// Première ligne utile d'un modèle : son objet, sinon le début de son corps.
  String _preview(EngineTemplate template) {
    final line = template.subject ?? template.body.split('\n').first;
    return line.length > 90 ? '${line.substring(0, 90)}…' : line;
  }

  Widget _row(EngineTemplate template) {
    final p = context.pal;
    return Container(
      key: ValueKey('template-row-${template.id}'),
      padding: const EdgeInsets.symmetric(vertical: 12),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: p.lineSoft)),
      ),
      child: Row(
        children: [
          Expanded(
            child: Hov(
              onTap: () => setState(() {
                editing = template;
                error = null;
              }),
              builder: (_, h) => Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    template.name,
                    style: Fonts.sans(size: 15, color: h ? p.accent : p.text),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    _preview(template),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: Fonts.sans(size: 13, color: p.dim),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(width: 14),
          IBtn(
            'trash',
            key: ValueKey('template-delete-${template.id}'),
            tip: t('Supprimer'),
            onTap: () => _confirmDelete(template),
          ),
        ],
      ),
    );
  }

  Widget _list() {
    final p = context.pal;
    final shown =
        error ?? (state.error == null ? null : engineErrorText(state.error!));
    return Padding(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            t('Modèles'),
            style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
          ),
          const SizedBox(height: 2),
          Text(
            t(
              'Textes réutilisables, avec des variables remplacées à l’insertion.',
            ),
            style: Fonts.sans(size: 13.5, color: p.dim),
          ),
          if (shown != null)
            Padding(
              padding: const EdgeInsets.only(top: 10),
              child: Text(
                shown,
                key: const ValueKey('template-error'),
                style: Fonts.sans(size: 13.5, color: p.accent),
              ),
            ),
          const SizedBox(height: 10),
          if (state.items.isEmpty)
            Text(
              t('Aucun modèle.'),
              style: Fonts.sans(size: 13.5, color: p.dim),
            ),
          for (final template in state.items) _row(template),
          const SizedBox(height: 10),
          Align(
            alignment: Alignment.centerLeft,
            child: Btn(
              key: const ValueKey('add-template'),
              icon: 'plus',
              label: t('Ajouter un modèle'),
              onTap: () => setState(() {
                creating = true;
                error = null;
              }),
            ),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    if (!creating && editing == null) return _list();
    return TemplateEditor(
      key: ValueKey('template-editor-${editing?.id ?? 'new'}'),
      template: editing,
      error: error,
      onSave: _save,
      onCancel: () => setState(() {
        creating = false;
        editing = null;
        error = null;
      }),
    );
  }
}
