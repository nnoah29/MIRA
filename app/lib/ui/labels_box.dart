import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../data/models.dart';
import '../i18n/i18n.dart';
import '../state/folder_map.dart';
import '../state/store.dart';
import '../theme/icons.dart';
import 'overlays.dart';
import '../theme/app_theme.dart';
import 'account_form_widgets.dart';
import 'widgets.dart';

/// Libellés d'un compte : renommer, supprimer, ajouter (boîtes du serveur, via le moteur).
class LabelsBox extends StatefulWidget {
  final String accountId;
  const LabelsBox({super.key, required this.accountId});

  @override
  State<LabelsBox> createState() => _LabelsBoxState();
}

class _LabelsBoxState extends State<LabelsBox> {
  final newName = TextEditingController();
  bool busy = false;

  @override
  void dispose() {
    newName.dispose();
    super.dispose();
  }

  Future<void> _add(MailStore s) async {
    final name = newName.text.trim();
    if (name.isEmpty) return;
    setState(() => busy = true);
    final ok = await s.createLabel(widget.accountId, name);
    if (!mounted) return;
    if (ok) newName.clear();
    setState(() => busy = false);
  }

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    if (s.source?.labelAdmin == null) return const SizedBox.shrink();
    final p = context.pal;
    final mine = [
      for (final e in s.source!.mappable.entries)
        if (e.value.account == widget.accountId) e,
    ];
    return Padding(
      padding: const EdgeInsets.only(bottom: 18),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Mono(
            t('LIBELLÉS'),
            size: 10.5,
            ls: .12,
            color: p.muted,
            height: null,
          ),
          const SizedBox(height: 4),
          Text(
            t('Dossiers du serveur : un message peut en porter plusieurs.'),
            style: Fonts.sans(size: 13, color: p.dim),
          ),
          const SizedBox(height: 10),
          Container(
            padding: const EdgeInsets.all(14),
            decoration: BoxDecoration(
              color: p.bg,
              border: Border.all(color: p.line),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                if (mine.isEmpty)
                  Padding(
                    padding: const EdgeInsets.only(bottom: 10),
                    child: Text(
                      t('Aucun libellé.'),
                      style: Fonts.sans(size: 13.5, color: p.dim),
                    ),
                  ),
                for (final e in mine)
                  _LabelRow(
                    key: ValueKey('label-${e.key}'),
                    id: e.key,
                    def: e.value,
                  ),
                _newRow(p, s),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _newRow(Pal p, MailStore s) => Row(
    children: [
      Container(
        width: 12,
        height: 12,
        decoration: BoxDecoration(border: Border.all(color: p.dim)),
      ),
      const SizedBox(width: 14),
      Expanded(
        child: FormField2(
          fieldKey: ValueKey('new-label-${widget.accountId}'),
          label: '',
          hint: t('Nouveau libellé'),
          controller: newName,
        ),
      ),
      const SizedBox(width: 12),
      Btn(
        key: ValueKey('add-label-${widget.accountId}'),
        icon: 'plus',
        label: t('Ajouter'),
        onTap: busy ? null : () => _add(s),
      ),
    ],
  );
}

class _LabelRow extends StatefulWidget {
  final String id;
  final LabelDef def;
  const _LabelRow({super.key, required this.id, required this.def});

  @override
  State<_LabelRow> createState() => _LabelRowState();
}

class _LabelRowState extends State<_LabelRow> {
  late final name = TextEditingController(text: widget.def.n);
  bool confirm = false, busy = false;

  @override
  void dispose() {
    name.dispose();
    super.dispose();
  }

  Future<void> _run(Future<bool> Function() job) async {
    setState(() => busy = true);
    await job();
    if (mounted) setState(() => busy = false);
  }

  @override
  Widget build(BuildContext context) {
    final s = context.read<MailStore>();
    return Padding(
      padding: const EdgeInsets.only(bottom: 10),
      child: Row(
        children: [
          Square(12, Color(widget.def.c)),
          const SizedBox(width: 14),
          if (widget.def.system)
            Expanded(
              child: Text(
                widget.def.n,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: Fonts.sans(size: 15, color: context.pal.text),
              ),
            )
          else
            Expanded(
              child: FormField2(
                fieldKey: ValueKey('label-name-${widget.id}'),
                onChanged: (_) => setState(() {}),
                label: '',
                controller: name,
              ),
            ),
          const SizedBox(width: 12),
          _MapButton(id: widget.id),
          const SizedBox(width: 8),
          if (!widget.def.system) ...[
            Btn(
              key: ValueKey('rename-label-${widget.id}'),
              label: t('Enregistrer'),
              onTap: busy || name.text.trim().isEmpty
                  ? null
                  : () => _run(() => s.renameLabel(widget.id, name.text)),
            ),
            const SizedBox(width: 8),
            Btn(
              key: ValueKey('delete-label-${widget.id}'),
              icon: 'trash',
              label: confirm ? t('Confirmer') : t('Supprimer'),
              onTap: busy
                  ? null
                  : confirm
                  ? () => _run(() => s.deleteLabel(widget.id))
                  : () => setState(() => confirm = true),
            ),
          ],
        ],
      ),
    );
  }
}

/// Bouton « Afficher comme » : choisit le dossier de l'interface auquel la boîte est rattachée.
class _MapButton extends StatelessWidget {
  final String id;
  const _MapButton({required this.id});

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final current = s.source?.folderMap[id];
    final shown = current == null ? t('Libellé') : t(folderById(current).n);
    return Builder(
      builder: (anchor) => Btn(
        key: ValueKey('map-label-$id'),
        label: '${t('Afficher comme')} : $shown ▾',
        on: current != null,
        onTap: () => _pick(anchor, s),
      ),
    );
  }

  void _pick(BuildContext anchor, MailStore s) {
    Pops.show(anchor, 'map-$id', (ctx, close) {
      final current = s.source?.folderMap[id];
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        mainAxisSize: MainAxisSize.min,
        children: [
          PopTitle(t('Afficher comme')),
          PopItem(
            key: ValueKey('map-$id-none'),
            leading: const SizedBox.shrink(),
            label: t('Libellé'),
            on: current == null,
            onTap: () {
              close();
              s.mapFolder(id, null);
            },
          ),
          for (final f in mappableFolders)
            PopItem(
              key: ValueKey('map-$id-$f'),
              leading: Ico(folderById(f).icon, size: 14),
              label: t(folderById(f).n),
              on: current == f,
              onTap: () {
                close();
                s.mapFolder(id, f);
              },
            ),
        ],
      );
    });
  }
}
