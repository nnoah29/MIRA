//! Choix du modèle d'une connexion : liste relevée chez le service, ou nom saisi à la main.

import 'package:flutter/material.dart';

import '../engine/ai_models.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Champ du modèle avec liste déroulante des modèles du service et bouton d'actualisation.
class AiModelPicker extends StatefulWidget {
  final String connectionId;
  final String? value;
  final List<AiModelInfo> models;
  final bool busy;
  final ValueChanged<String> onChanged;
  final VoidCallback onRefresh;
  const AiModelPicker({
    super.key,
    required this.connectionId,
    required this.value,
    required this.models,
    required this.busy,
    required this.onChanged,
    required this.onRefresh,
  });

  @override
  State<AiModelPicker> createState() => _AiModelPickerState();
}

class _AiModelPickerState extends State<AiModelPicker> {
  late final TextEditingController ctl = TextEditingController(
    text: widget.value ?? '',
  );
  final FocusNode focus = FocusNode();
  late String committed = widget.value ?? '';
  bool open = false;

  @override
  void initState() {
    super.initState();
    focus.addListener(() {
      if (!focus.hasFocus) _commit(ctl.text);
    });
  }

  void _commit(String text) {
    final value = text.trim();
    if (value == committed) return;
    committed = value;
    widget.onChanged(value);
  }

  @override
  void didUpdateWidget(AiModelPicker old) {
    super.didUpdateWidget(old);
    if (old.value != widget.value) {
      committed = widget.value ?? '';
      if (committed != ctl.text) ctl.text = committed;
    }
  }

  @override
  void dispose() {
    focus.dispose();
    ctl.dispose();
    super.dispose();
  }

  void _pick(String id) {
    ctl.text = id;
    setState(() => open = false);
    _commit(id);
  }

  Widget _field(Pal p) => TextField(
    key: ValueKey('ai-model-field-${widget.connectionId}'),
    controller: ctl,
    focusNode: focus,
    cursorColor: p.accent,
    style: Fonts.sans(size: 14, color: p.text),
    onSubmitted: _commit,
    decoration: InputDecoration(
      isDense: true,
      hintText: t('Nom du modèle'),
      hintStyle: Fonts.sans(size: 14, color: p.dim),
      border: OutlineInputBorder(borderSide: BorderSide(color: p.line)),
      enabledBorder: OutlineInputBorder(borderSide: BorderSide(color: p.line)),
      focusedBorder: OutlineInputBorder(
        borderSide: BorderSide(color: p.accent),
      ),
    ),
  );

  Widget _entry(AiModelInfo model, Pal p) => Hov(
    key: ValueKey('ai-model-${widget.connectionId}-${model.id}'),
    onTap: () => _pick(model.id),
    builder: (_, h) => Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      color: h ? p.surface2 : null,
      child: Text(
        model.id,
        style: Fonts.sans(
          size: 13.5,
          color: model.id == widget.value ? p.accent : p.text,
        ),
      ),
    ),
  );

  Widget _list(Pal p) => Container(
    constraints: const BoxConstraints(maxHeight: 220),
    margin: const EdgeInsets.only(top: 6),
    decoration: BoxDecoration(border: Border.all(color: p.line)),
    child: widget.models.isEmpty
        ? Padding(
            padding: const EdgeInsets.all(12),
            child: Text(
              t('Aucun modèle relevé : saisis son nom, ou actualise la liste.'),
              style: Fonts.sans(size: 13, color: p.dim),
            ),
          )
        : ListView(
            shrinkWrap: true,
            children: [for (final m in widget.models) _entry(m, p)],
          ),
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            Expanded(child: _field(p)),
            const SizedBox(width: 8),
            IBtn(
              'chev',
              key: ValueKey('ai-model-menu-${widget.connectionId}'),
              tip: t('Modèles du service'),
              onTap: () => setState(() => open = !open),
            ),
            IBtn(
              'refresh',
              key: ValueKey('ai-model-refresh-${widget.connectionId}'),
              tip: t('Actualiser la liste'),
              onTap: widget.busy ? null : widget.onRefresh,
            ),
          ],
        ),
        if (open) _list(p),
      ],
    );
  }
}
