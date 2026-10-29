//! Confidentialité d'une connexion d'IA : contenus autorisés et masquage des données sensibles.

import 'package:flutter/material.dart';

import '../engine/ai_models.dart';
import '../i18n/i18n.dart';
import '../state/ai_settings_state.dart';
import '../theme/app_theme.dart';
import 'ai_kind_names.dart';
import 'rule_widgets.dart';
import 'widgets.dart';

/// Nom affiché d'une catégorie de données sensibles.
String aiCategoryName(String category) => switch (category) {
  'iban' => t('IBAN'),
  'card' => t('Carte bancaire'),
  'secret' => t('Mot de passe ou clé'),
  'otp' => t('Code à usage unique'),
  'email' => t('Adresse e-mail'),
  'phone' => t('Téléphone'),
  _ => category,
};

/// Réglages de confidentialité d'une connexion ; chaque changement est enregistré par [state].
class AiPrivacySection extends StatelessWidget {
  final AiSettingsState state;
  final AiConnectionInfo connection;
  const AiPrivacySection({
    super.key,
    required this.state,
    required this.connection,
  });

  String get id => connection.id;

  Widget _label(Pal p, String text) =>
      Mono(text, size: 10.5, ls: .12, color: p.muted, height: null);

  void _toggleKind(String kind) {
    final kept = [...connection.allowedKinds];
    kept.contains(kind) ? kept.remove(kind) : kept.add(kind);
    state.setAllowedContent(connection, kept);
  }

  void _toggleCategory(String category) {
    final masking = connection.masking;
    final kept = [...masking.categories];
    kept.contains(category) ? kept.remove(category) : kept.add(category);
    final ordered = [
      for (final c in aiMaskCategories)
        if (kept.contains(c)) c,
    ];
    state.setMasking(connection, masking.copyWith(categories: ordered));
  }

  void _toggleMasking(bool on) => state.setMasking(
    connection,
    connection.masking.copyWith(mode: on ? 'mask' : 'off'),
  );

  Widget _kinds(Pal p) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      _label(p, t('Contenu autorisé')),
      const SizedBox(height: 6),
      Wrap(
        spacing: 8,
        runSpacing: 8,
        children: [
          for (final kind in aiKnownKinds)
            RuleChip(
              key: kind == aiKindMail ? ValueKey('ai-privacy-kinds-$id') : null,
              label: aiKindName(kind),
              selected: connection.allowedKinds.contains(kind),
              onTap: () => _toggleKind(kind),
            ),
        ],
      ),
    ],
  );

  Widget _switchRow(Pal p) {
    final on = connection.masking.on;
    return Row(
      children: [
        RuleToggle(
          key: ValueKey('ai-privacy-mask-$id'),
          value: on,
          onChanged: _toggleMasking,
        ),
        const SizedBox(width: 12),
        Expanded(
          child: Text(
            on ? t('Masquées avant l’envoi') : t('Non masquées'),
            style: Fonts.sans(size: 15, color: p.text),
          ),
        ),
      ],
    );
  }

  Widget _categories() => Wrap(
    spacing: 8,
    runSpacing: 8,
    children: [
      for (final category in aiMaskCategories)
        RuleChip(
          key: ValueKey('ai-privacy-cat-$id-$category'),
          label: aiCategoryName(category),
          selected: connection.masking.categories.contains(category),
          onTap: () => _toggleCategory(category),
        ),
    ],
  );

  Widget _masking(Pal p) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      _label(p, t('Données sensibles')),
      const SizedBox(height: 6),
      _switchRow(p),
      if (connection.masking.on) ...[const SizedBox(height: 10), _categories()],
      const SizedBox(height: 10),
      Text(
        t('Détection au mieux : elle peut se tromper.'),
        key: ValueKey('ai-privacy-notice-$id'),
        style: Fonts.sans(size: 13, color: p.dim),
      ),
    ],
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final busy = state.busy.contains(id);
    return AbsorbPointer(
      key: ValueKey('ai-privacy-$id'),
      absorbing: busy,
      child: Opacity(
        opacity: busy ? .5 : 1,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [_kinds(p), const SizedBox(height: 16), _masking(p)],
        ),
      ),
    );
  }
}
