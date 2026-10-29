//! Contenu du popover des modèles : recherche, liste du service, actualisation, saisie libre d'un nom.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../engine/ai_models.dart';
import '../i18n/i18n.dart';
import '../state/ai_settings_state.dart';
import '../theme/app_theme.dart';
import 'overlays.dart';

/// Popover d'une connexion : choisir un modèle de la liste ou taper un autre nom.
class AiModelPopup extends StatefulWidget {
  final AiSettingsState state;
  final String connectionId;
  final VoidCallback close;
  const AiModelPopup({
    super.key,
    required this.state,
    required this.connectionId,
    required this.close,
  });

  @override
  State<AiModelPopup> createState() => _AiModelPopupState();
}

class _AiModelPopupState extends State<AiModelPopup> {
  final search = TextEditingController();
  final typed = TextEditingController();
  bool freeText = false;

  String get id => widget.connectionId;

  AiConnectionInfo? get _connection {
    for (final c in widget.state.connections) {
      if (c.id == id) return c;
    }
    return null;
  }

  @override
  void dispose() {
    search.dispose();
    typed.dispose();
    super.dispose();
  }

  Future<void> _choose(String model) async {
    final c = _connection;
    widget.close();
    final name = model.trim();
    if (c == null || name.isEmpty || name == c.defaultModel) return;
    await widget.state.setModel(c, name);
  }

  Future<void> _refresh() async {
    if (widget.state.busy.contains(id)) return;
    await widget.state.loadModels(id, refresh: true);
  }

  InputDecoration _decoration(Pal p, String hint) => InputDecoration(
    isDense: true,
    hintText: hint,
    hintStyle: Fonts.sans(size: 14, color: p.dim),
    border: OutlineInputBorder(borderSide: BorderSide(color: p.line)),
    enabledBorder: OutlineInputBorder(borderSide: BorderSide(color: p.line)),
    focusedBorder: OutlineInputBorder(borderSide: BorderSide(color: p.accent)),
  );

  Widget _field(Pal p, Key key, TextEditingController ctl, String hint) =>
      Padding(
        padding: const EdgeInsets.fromLTRB(6, 2, 6, 6),
        child: TextField(
          key: key,
          controller: ctl,
          autofocus: true,
          cursorColor: p.accent,
          style: Fonts.sans(size: 14, color: p.text),
          decoration: _decoration(p, hint),
          onChanged: (_) => setState(() {}),
          onSubmitted: freeText ? _choose : null,
        ),
      );

  Widget _entry(AiModelInfo m, String? current) => PopItem(
    key: ValueKey('ai-model-$id-${m.id}'),
    leading: const SizedBox.shrink(),
    label: m.id,
    on: m.id == current,
    onTap: () => _choose(m.id),
  );

  Widget _note(Pal p, String text) => Padding(
    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
    child: Text(text, style: Fonts.sans(size: 13, color: p.dim)),
  );

  Widget _list(Pal p, AiConnectionInfo c) {
    final all = widget.state.modelsOf(c);
    final q = search.text.trim().toLowerCase();
    final shown = [
      for (final m in all)
        if (m.id.toLowerCase().contains(q)) m,
    ];
    if (all.isEmpty) {
      return _note(
        p,
        t('Aucun modèle relevé : saisis son nom, ou actualise la liste.'),
      );
    }
    if (shown.isEmpty) return _note(p, t('Aucun modèle ne correspond.'));
    return ConstrainedBox(
      constraints: const BoxConstraints(maxHeight: 240),
      child: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [for (final m in shown) _entry(m, c.defaultModel)],
        ),
      ),
    );
  }

  Widget _actions(AiConnectionInfo c) {
    final busy = widget.state.busy.contains(id);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const PopSep(),
        PopItem(
          key: ValueKey('ai-model-refresh-$id'),
          leading: popIco('refresh', context),
          label: busy ? t('Actualisation…') : t('Actualiser la liste'),
          onTap: _refresh,
        ),
        PopItem(
          key: ValueKey('ai-model-custom-$id'),
          leading: popIco('pen', context),
          label: t('Saisir un autre nom…'),
          onTap: () => setState(() => freeText = true),
        ),
      ],
    );
  }

  Widget _body(Pal p, AiConnectionInfo c) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    mainAxisSize: MainAxisSize.min,
    children: freeText
        ? [
            PopTitle(t('Nom du modèle')),
            _field(
              p,
              ValueKey('ai-model-field-$id'),
              typed,
              t('Nom du modèle'),
            ),
          ]
        : [
            PopTitle(t('Modèles du service')),
            _field(
              p,
              ValueKey('ai-model-search-$id'),
              search,
              t('Rechercher un modèle'),
            ),
            _list(p, c),
            _actions(c),
          ],
  );

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: widget.state,
    builder: (context, _) {
      final c = _connection;
      if (c == null) return const SizedBox.shrink();
      return CallbackShortcuts(
        bindings: {
          const SingleActivator(LogicalKeyboardKey.escape): widget.close,
        },
        child: FocusScope(
          child: SizedBox(width: 300, child: _body(context.pal, c)),
        ),
      );
    },
  );
}
