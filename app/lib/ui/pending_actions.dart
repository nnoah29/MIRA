import 'dart:async';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../engine/account_admin.dart';
import '../i18n/i18n.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Nom lisible d'une action en file.
String actionLabel(String kind) => switch (kind) {
  'mark_read' => t('Marquer lu / non lu'),
  'mark_flagged' => t('Favori'),
  'archive' => t('Archiver'),
  'trash' => t('Mettre à la corbeille'),
  'delete_permanently' => t('Supprimer définitivement'),
  'mark_spam' => t('Spam'),
  'move' => t('Déplacer'),
  'empty_mailbox' => t('Vider un dossier'),
  'save_draft' => t('Copier un brouillon'),
  _ => kind,
};

String _stateLabel(String state) => switch (state) {
  'failed' => t('échec'),
  'uncertain' => t('incertain'),
  'in_progress' => t('en cours'),
  _ => t('en attente'),
};

/// Actions du compte pas encore appliquées sur le serveur : relancer ou abandonner.
class PendingActions extends StatefulWidget {
  final AccountDetail account;
  final AccountAdmin admin;
  const PendingActions({super.key, required this.account, required this.admin});

  @override
  State<PendingActions> createState() => _PendingActionsState();
}

class _PendingActionsState extends State<PendingActions> {
  List<QueuedAction>? items;
  bool busy = false;

  @override
  void initState() {
    super.initState();
    unawaited(_load());
  }

  Future<void> _load() async {
    try {
      final got = await widget.admin.pendingActions(widget.account.id);
      if (mounted) setState(() => items = got);
    } catch (_) {
      if (mounted) setState(() => items = const []);
    }
  }

  Future<void> _do(Future<void> Function() job, String done) async {
    final s = context.read<MailStore>();
    setState(() => busy = true);
    try {
      await job();
      s.toast(done);
    } catch (_) {
      s.toast(t('Opération impossible'), err: true);
    }
    await _load();
    if (mounted) setState(() => busy = false);
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final list = items;
    if (list == null) return Mono(t('Chargement…'), color: p.dim, height: null);
    if (list.isEmpty) {
      return Text(
        t('Tout est à jour sur le serveur.'),
        style: Fonts.sans(size: 13.5, color: p.dim),
      );
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        for (final a in list)
          Padding(
            key: ValueKey('queued-${a.id}'),
            padding: const EdgeInsets.only(bottom: 8),
            child: Row(
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        '${actionLabel(a.kind)} · ${tn(a.targets, 'message', 'messages')}',
                        style: Fonts.sans(size: 14, color: p.text),
                      ),
                      Mono(
                        [
                          _stateLabel(a.state),
                          if (a.attempts > 0) t('{0} essais', [a.attempts]),
                          ?a.error,
                        ].join(' · '),
                        size: 10.5,
                        height: null,
                        color: a.state == 'failed' ? p.accent : p.dim,
                      ),
                    ],
                  ),
                ),
                if (a.state == 'failed' || a.state == 'uncertain') ...[
                  Btn(
                    label: t('Réessayer'),
                    onTap: busy
                        ? null
                        : () => _do(
                            () => widget.admin.retryAction(a.id),
                            t('Action relancée'),
                          ),
                  ),
                  const SizedBox(width: 8),
                ],
                Btn(
                  key: ValueKey('queued-cancel-${a.id}'),
                  label: t('Abandonner'),
                  onTap: busy || a.state == 'in_progress'
                      ? null
                      : () => _do(
                          () => widget.admin.cancelAction(a.id),
                          t('Action abandonnée'),
                        ),
                ),
              ],
            ),
          ),
      ],
    );
  }
}
