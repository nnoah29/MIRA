import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../data/models.dart';
import '../i18n/i18n.dart';
import '../state/folder_map.dart';
import '../state/store.dart';
import '../theme/icons.dart';
import 'overlays.dart';
import 'widgets.dart';

/// Bouton « Afficher comme » : choisit le dossier de l'interface auquel la boîte est rattachée.
class LabelMapButton extends StatelessWidget {
  final String id;
  const LabelMapButton({super.key, required this.id});

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
