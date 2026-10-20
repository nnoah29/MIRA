import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../i18n/i18n.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import 'account_form_widgets.dart';
import 'widgets.dart';

/// Libellés locaux : communs à tous les comptes, jamais envoyés au serveur.
class LocalLabelsSection extends StatefulWidget {
  const LocalLabelsSection({super.key});

  @override
  State<LocalLabelsSection> createState() => _LocalLabelsSectionState();
}

class _LocalLabelsSectionState extends State<LocalLabelsSection> {
  final ctl = TextEditingController();
  bool invalid = false;

  @override
  void dispose() {
    ctl.dispose();
    super.dispose();
  }

  void _add(MailStore s) {
    final ok = s.addLocalLabel(ctl.text);
    setState(() => invalid = !ok);
    if (ok) ctl.clear();
  }

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    final defs = s.localLabels.defs;
    return Padding(
      padding: const EdgeInsets.fromLTRB(20, 16, 20, 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            t('Libellés locaux'),
            style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
          ),
          Text(
            t(
              'Rangés sur cet appareil seulement, valables pour tous les comptes. Le serveur n’en sait rien.',
            ),
            style: Fonts.sans(size: 13.5, color: p.dim),
          ),
          const SizedBox(height: 12),
          if (defs.isEmpty)
            Text(
              t('Aucun libellé local.'),
              style: Fonts.sans(size: 13.5, color: p.dim),
            ),
          for (final e in defs.entries) _LocalRow(id: e.key, store: s),
          const SizedBox(height: 12),
          Wrap(
            spacing: 12,
            runSpacing: 10,
            crossAxisAlignment: WrapCrossAlignment.center,
            children: [
              SizedBox(
                width: 280,
                child: Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 12,
                    vertical: 10,
                  ),
                  decoration: BoxDecoration(
                    color: p.bg,
                    border: Border.all(color: invalid ? p.accent : p.line),
                  ),
                  child: TextField(
                    key: const ValueKey('local-label-input'),
                    controller: ctl,
                    onSubmitted: (_) => _add(s),
                    cursorColor: p.accent,
                    cursorWidth: 1,
                    style: Fonts.sans(size: 15, color: p.text),
                    decoration: InputDecoration(
                      isDense: true,
                      border: InputBorder.none,
                      contentPadding: EdgeInsets.zero,
                      hintText: t('Nouveau libellé local'),
                      hintStyle: Fonts.sans(size: 15, color: p.dim),
                    ),
                  ),
                ),
              ),
              Btn(
                key: const ValueKey('add-local-label'),
                icon: 'plus',
                label: t('Ajouter'),
                onTap: () => _add(s),
              ),
            ],
          ),
          if (invalid)
            Padding(
              padding: const EdgeInsets.only(top: 8),
              child: Text(
                t('Saisis un nom, différent de ceux déjà utilisés.'),
                style: Fonts.sans(size: 13, color: p.accent),
              ),
            ),
        ],
      ),
    );
  }
}

class _LocalRow extends StatefulWidget {
  final String id;
  final MailStore store;
  const _LocalRow({required this.id, required this.store});

  @override
  State<_LocalRow> createState() => _LocalRowState();
}

class _LocalRowState extends State<_LocalRow> {
  late final name = TextEditingController(
    text: widget.store.localLabels.defs[widget.id]?.n,
  );
  bool confirm = false;

  @override
  void dispose() {
    name.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final s = widget.store;
    final def = s.localLabels.defs[widget.id];
    if (def == null) return const SizedBox.shrink();
    final uses = s.localLabels.count(widget.id);
    return Padding(
      padding: const EdgeInsets.only(bottom: 10),
      child: Row(
        children: [
          Tooltip(
            message: t('Changer la couleur'),
            child: InkWell(
              key: ValueKey('local-color-${widget.id}'),
              onTap: () => s.recolorLocalLabel(widget.id),
              child: Square(14, Color(def.c)),
            ),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: FormField2(
              fieldKey: ValueKey('local-name-${widget.id}'),
              label: '',
              controller: name,
              onChanged: (_) => setState(() {}),
            ),
          ),
          const SizedBox(width: 12),
          Mono(
            tn(uses, 'conversation', 'conversations'),
            size: 10.5,
            ls: .08,
            color: context.pal.muted,
            height: null,
          ),
          const SizedBox(width: 12),
          Btn(
            key: ValueKey('local-rename-${widget.id}'),
            label: t('Enregistrer'),
            onTap: name.text.trim() == def.n
                ? null
                : () => s.renameLocalLabel(widget.id, name.text),
          ),
          const SizedBox(width: 8),
          Btn(
            key: ValueKey('local-delete-${widget.id}'),
            icon: 'trash',
            label: confirm ? t('Confirmer') : t('Supprimer'),
            onTap: confirm
                ? () => s.deleteLocalLabel(widget.id)
                : () => setState(() => confirm = true),
          ),
        ],
      ),
    );
  }
}
