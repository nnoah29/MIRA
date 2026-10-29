//! Bandeau d'onglets des fournisseurs d'IA : un onglet par fournisseur, une pastille s'il a une connexion.

import 'package:flutter/material.dart';

import '../engine/ai_models.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'ai_connection_card.dart';
import 'widgets.dart';

/// Onglets des fournisseurs, dans l'ordre du moteur ; même style que ceux des réglages.
class AiProviderTabs extends StatelessWidget {
  final List<AiProviderInfo> providers;
  final List<AiConnectionInfo> connections;
  final String selected;
  final ValueChanged<String> onSelect;
  const AiProviderTabs({
    super.key,
    required this.providers,
    required this.connections,
    required this.selected,
    required this.onSelect,
  });

  /// Pastille d'un onglet : pleine et colorée pour la connexion par défaut, creuse pour une autre.
  Widget _dot(Pal p, AiProviderInfo provider) {
    final mine = connections.where((c) => c.provider == provider.id);
    if (mine.isEmpty) return const SizedBox.shrink();
    final isDefault = mine.any((c) => c.isDefault);
    final dot = Container(
      width: 7,
      height: 7,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        color: isDefault ? p.accent : null,
        border: Border.all(color: isDefault ? p.accent : p.muted),
      ),
    );
    return Padding(
      padding: const EdgeInsets.only(left: 7),
      child: isDefault
          ? Tooltip(
              key: ValueKey('ai-tab-default-${provider.id}'),
              message: t('Par défaut'),
              child: dot,
            )
          : KeyedSubtree(
              key: ValueKey('ai-tab-dot-${provider.id}'),
              child: dot,
            ),
    );
  }

  Widget _tab(Pal p, AiProviderInfo provider) => Hov(
    key: ValueKey('ai-tab-${provider.id}'),
    onTap: () => onSelect(provider.id),
    builder: (_, h) {
      final on = provider.id == selected;
      return AnimatedContainer(
        duration: const Duration(milliseconds: 130),
        padding: const EdgeInsets.fromLTRB(2, 10, 2, 11),
        margin: const EdgeInsets.only(right: 28),
        decoration: BoxDecoration(
          border: Border(
            bottom: BorderSide(
              color: on ? p.accent : Colors.transparent,
              width: 2,
            ),
          ),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Mono(
              aiProviderTitle(provider),
              ls: .14,
              color: on ? p.accent : (h ? p.text : p.muted),
              height: null,
            ),
            _dot(p, provider),
          ],
        ),
      );
    },
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Container(
      margin: const EdgeInsets.only(top: 16),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: p.line)),
      ),
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: Row(children: [for (final v in providers) _tab(p, v)]),
      ),
    );
  }
}
