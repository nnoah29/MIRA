//! « Écrire avec l'IA » dans le composeur : barre d'instruction avec réponse, erreur et annulation, et actions sur la sélection.

import 'package:flutter/material.dart';

import '../i18n/i18n.dart';
import '../state/draft_ai_controller.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'widgets.dart';

/// Barre au-dessus de la barre d'outils du composeur : on écrit ce qu'on veut, l'IA remplit le brouillon, on peut annuler.
class ComposerAiBar extends StatefulWidget {
  final DraftAiController ai;
  final VoidCallback onClose;
  const ComposerAiBar({super.key, required this.ai, required this.onClose});

  @override
  State<ComposerAiBar> createState() => _ComposerAiBarState();
}

class _ComposerAiBarState extends State<ComposerAiBar> {
  final input = TextEditingController();
  final focus = FocusNode();

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) focus.requestFocus();
    });
  }

  @override
  void dispose() {
    input.dispose();
    focus.dispose();
    super.dispose();
  }

  void _send() {
    final text = input.text;
    if (text.trim().isEmpty || widget.ai.busy) return;
    input.clear();
    widget.ai.run(text);
  }

  Widget _note(Pal p, String key, String text, {bool error = false}) => Padding(
    padding: const EdgeInsets.only(top: 8),
    child: Text(
      text,
      key: ValueKey(key),
      style: Fonts.sans(size: 13.5, color: error ? p.accent : p.dim),
    ),
  );

  Widget _field(Pal p, DraftAiController ai) => Row(
    children: [
      Ico('spark', size: 14, color: p.accent),
      const SizedBox(width: 10),
      Expanded(
        child: TextField(
          key: const ValueKey('composer-ai-input'),
          controller: input,
          focusNode: focus,
          enabled: !ai.busy,
          cursorColor: p.accent,
          style: Fonts.sans(size: 15, color: p.text),
          onSubmitted: (_) => _send(),
          decoration: InputDecoration(
            isDense: true,
            border: InputBorder.none,
            hintText: ai.busy ? t('L’IA écrit…') : t('Que veux-tu écrire ?'),
            hintStyle: Fonts.sans(size: 15, color: p.dim),
          ),
        ),
      ),
      if (ai.busy)
        Btn(
          key: const ValueKey('composer-ai-stop'),
          icon: 'x',
          label: t('Arrêter'),
          onTap: ai.stop,
        )
      else
        IBtn(
          'send',
          key: const ValueKey('composer-ai-send'),
          tip: t('Envoyer à l’IA'),
          onTap: _send,
        ),
      IBtn(
        'x',
        key: const ValueKey('composer-ai-close'),
        tip: t('Fermer'),
        onTap: widget.onClose,
      ),
    ],
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return ListenableBuilder(
      listenable: widget.ai,
      builder: (context, _) {
        final ai = widget.ai;
        return Container(
          key: const ValueKey('composer-ai-bar'),
          padding: const EdgeInsets.fromLTRB(16, 10, 8, 12),
          decoration: BoxDecoration(
            color: p.surface,
            border: Border(top: BorderSide(color: p.lineSoft)),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _field(p, ai),
              if (ai.reply != null && ai.reply!.trim().isNotEmpty)
                _note(p, 'composer-ai-reply', ai.reply!),
              if (ai.error != null)
                _note(p, 'composer-ai-error', ai.error!, error: true),
              if (ai.summary != null)
                _note(
                  p,
                  'composer-ai-changes',
                  t('Modifié par l’IA : {0}', [ai.summary!]),
                ),
              if (ai.canUndo)
                Padding(
                  padding: const EdgeInsets.only(top: 8),
                  child: Align(
                    alignment: Alignment.centerLeft,
                    child: Btn(
                      key: const ValueKey('composer-ai-undo'),
                      icon: 'arrowl',
                      label: t('Annuler la modification de l’IA'),
                      onTap: ai.undo,
                    ),
                  ),
                ),
            ],
          ),
        );
      },
    );
  }
}

/// Actions de l'IA sur le passage sélectionné, sous la barre de formatage flottante.
class ComposerAiActions extends StatelessWidget {
  final void Function(String instruction) onAction;
  const ComposerAiActions({super.key, required this.onAction});

  Widget _chip(Pal p, String id, String label, String instruction) => Hov(
    key: ValueKey('composer-ai-action-$id'),
    onTap: () => onAction(instruction),
    builder: (_, h) => Container(
      padding: const EdgeInsets.symmetric(horizontal: 9, vertical: 6),
      color: h ? p.surface : null,
      child: Mono(label, color: h ? p.accent : p.muted, height: null),
    ),
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Padding(
          padding: const EdgeInsets.only(left: 6, right: 2),
          child: Ico('spark', size: 13, color: p.accent),
        ),
        _chip(p, 'reformulate', t('Reformuler'), t('Reformule ce passage.')),
        _chip(p, 'shorten', t('Raccourcir'), t('Raccourcis ce passage.')),
        _chip(
          p,
          'fix',
          t('Corriger'),
          t('Corrige l’orthographe et la grammaire de ce passage.'),
        ),
        _chip(
          p,
          'translate',
          t('Traduire'),
          t('Traduis ce passage en anglais.'),
        ),
      ],
    );
  }
}
