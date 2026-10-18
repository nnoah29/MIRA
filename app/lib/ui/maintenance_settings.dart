import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../data/format.dart';
import '../engine/maintenance_admin.dart';
import '../i18n/i18n.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Stockage et entretien du moteur : volumes, vérification, purge du cache, index, exports.
class MaintenanceSettings extends StatefulWidget {
  const MaintenanceSettings({super.key});

  @override
  State<MaintenanceSettings> createState() => _MaintenanceSettingsState();
}

class _MaintenanceSettingsState extends State<MaintenanceSettings> {
  StorageStats? stats;
  bool busy = false;

  @override
  void initState() {
    super.initState();
    _refresh();
  }

  Future<void> _refresh() async {
    final admin = context.read<MailStore>().source?.maintenance;
    if (admin == null) return;
    try {
      final fresh = await admin.stats();
      if (mounted) setState(() => stats = fresh);
    } catch (_) {}
  }

  Future<void> _run(String done, Future<String?> Function() job) async {
    final s = context.read<MailStore>();
    setState(() => busy = true);
    try {
      final msg = await job();
      s.toast(msg ?? done);
    } catch (_) {
      s.toast(t('Opération impossible'), err: true);
    }
    if (mounted) setState(() => busy = false);
    await _refresh();
  }

  Future<String?> _export(MailStore s, String op, String name) async {
    final path = await s.pickExportPath(name);
    if (path == null) return t('Export annulé');
    final bytes = await s.source!.maintenance!.export(op, path);
    return t('Exporté ({0})', [sizeOf(bytes)]);
  }

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    final admin = s.source?.maintenance;
    final st = stats;
    Widget action(String label, String hint, Future<String?> Function() job) =>
        Padding(
          padding: const EdgeInsets.only(top: 14),
          child: Row(
            children: [
              Expanded(
                child: Text(hint, style: Fonts.sans(size: 13.5, color: p.dim)),
              ),
              const SizedBox(width: 16),
              Btn(label: label, onTap: busy ? null : () => _run(label, job)),
            ],
          ),
        );
    return Padding(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            t('Stockage et maintenance'),
            style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
          ),
          const SizedBox(height: 6),
          Text(
            st == null
                ? t('Mesure en cours…')
                : t(
                    '{0} messages, {1} pièces jointes ({2} en cache), {3} sur disque',
                    [
                      st.messages,
                      st.attachments,
                      st.cached,
                      sizeOf(st.compressedBytes),
                    ],
                  ),
            key: const ValueKey('storage-summary'),
            style: Fonts.sans(size: 13.5, color: p.dim),
          ),
          if (admin != null) ...[
            action(
              t('Vérifier la base'),
              t('Contrôle l’intégrité de la base locale.'),
              () async {
                final ok = await admin.checkIntegrity();
                return ok
                    ? t('Base saine')
                    : t('Base endommagée : exporte un diagnostic');
              },
            ),
            action(
              t('Vider le cache'),
              t(
                'Supprime les pièces jointes gardées en local ; elles seront retéléchargées au besoin.',
              ),
              () async {
                await admin.purgeAttachments();
                return t('Cache vidé');
              },
            ),
            action(
              t('Reconstruire l’index'),
              t('Reconstruit la recherche locale si des résultats manquent.'),
              () async {
                await admin.rebuildIndex();
                return t('Index reconstruit');
              },
            ),
            action(
              t('Exporter un diagnostic'),
              t(
                'Écrit un rapport sans contenu de messages ni secrets, utile pour signaler un problème.',
              ),
              () => _export(s, 'export_diagnostic', 'mira-diagnostic.json'),
            ),
            action(
              t('Exporter le profil'),
              t('Écrit la configuration des comptes (sans mots de passe).'),
              () => _export(s, 'export_profile', 'mira-profil.json'),
            ),
          ],
        ],
      ),
    );
  }
}
