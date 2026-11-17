//! Réglages de la dictée vocale : langue, et un modèle par langue à télécharger, suivre ou supprimer.

import 'package:flutter/material.dart';

import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import '../voice/dictation_types.dart';
import '../voice/voice_model_catalog.dart';
import '../voice/voice_model_store.dart';
import '../voice/voice_services.dart';
import 'ai_local_text.dart';
import 'voice_model_dialog.dart';
import 'widgets.dart';

/// Section « Dictée vocale » ; vide quand l'app n'a pas de services de dictée.
class VoiceSettings extends StatelessWidget {
  const VoiceSettings({super.key});

  @override
  Widget build(BuildContext context) {
    final services = voiceServicesOf(context);
    if (services == null) return const SizedBox.shrink();
    return ListenableBuilder(
      listenable: Listenable.merge([services.models, services.dictation]),
      builder: (context, _) => _Content(services),
    );
  }
}

class _Content extends StatelessWidget {
  const _Content(this.services);

  final VoiceServices services;

  /// Change la langue de dictée et la mémorise.
  Future<void> _pick(DictationLanguage language) async {
    await services.dictation.setLanguage(language);
    await services.preferences.setLanguage(language);
  }

  /// Un bouton par langue, celui de la langue active est enfoncé.
  Widget _languages() => Wrap(
    spacing: 8,
    children: [
      for (final l in DictationLanguage.values)
        Btn(
          key: ValueKey('voice-lang-${l.code}'),
          label: l.label,
          on: services.dictation.language == l,
          onTap: () => _pick(l),
        ),
    ],
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Padding(
      key: const ValueKey('voice-settings'),
      padding: const EdgeInsets.fromLTRB(20, 16, 20, 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            t('Dictée vocale'),
            style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
          ),
          Text(
            t(
              'Dicte tes messages et tes demandes à l’IA. La reconnaissance tourne sur ta machine : ta voix ne part jamais sur Internet.',
            ),
            style: Fonts.sans(size: 13.5, color: p.dim),
          ),
          const SizedBox(height: 12),
          Text(
            t('Langue de dictée'),
            style: Fonts.sans(size: 13.5, color: p.muted),
          ),
          const SizedBox(height: 6),
          _languages(),
          const SizedBox(height: 14),
          for (final entry in services.models.catalog)
            _ModelCard(services: services, entry: entry),
          Text(
            t(
              'Les noms propres rares et les mots d’une autre langue peuvent être mal reconnus.',
            ),
            style: Fonts.sans(size: 13, color: p.muted),
          ),
        ],
      ),
    );
  }
}

class _ModelCard extends StatelessWidget {
  const _ModelCard({required this.services, required this.entry});

  final VoiceServices services;
  final VoiceModelEntry entry;

  VoiceModelStore get _store => services.models;

  /// Demande confirmation puis lance le téléchargement.
  Future<void> _install(BuildContext context) async {
    if (await confirmVoiceModelInstall(context, entry)) {
      _store.install(entry.language);
    }
  }

  /// Zone d'état et d'actions selon l'avancement du modèle.
  Widget _state(BuildContext context, Pal p) {
    final code = entry.language.code;
    return switch (_store.statusOf(entry.language)) {
      VoiceModelNotInstalled() => Btn(
        key: ValueKey('voice-install-$code'),
        label: t('Télécharger'),
        onTap: () => _install(context),
      ),
      VoiceModelDownloading(:final progress) => Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            '${(progress * 100).round()} %',
            style: Fonts.sans(size: 13, color: p.dim),
          ),
          const SizedBox(width: 10),
          Btn(
            key: ValueKey('voice-cancel-$code'),
            label: t('Annuler'),
            onTap: () => _store.cancel(entry.language),
          ),
        ],
      ),
      VoiceModelInstalled() => Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            t('Installé'),
            key: ValueKey('voice-installed-$code'),
            style: Fonts.sans(size: 13, color: p.dim),
          ),
          const SizedBox(width: 10),
          Btn(
            key: ValueKey('voice-remove-$code'),
            label: t('Supprimer'),
            onTap: () => _store.remove(entry.language),
          ),
        ],
      ),
      VoiceModelFailed() => Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            t('Téléchargement impossible'),
            key: ValueKey('voice-failed-$code'),
            style: Fonts.sans(size: 13, color: p.accent),
          ),
          const SizedBox(width: 10),
          Btn(
            key: ValueKey('voice-install-$code'),
            label: t('Réessayer'),
            onTap: () => _install(context),
          ),
        ],
      ),
    };
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Container(
      key: ValueKey('voice-model-${entry.language.code}'),
      margin: const EdgeInsets.only(bottom: 10),
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(border: Border.all(color: p.line)),
      child: Wrap(
        alignment: WrapAlignment.spaceBetween,
        crossAxisAlignment: WrapCrossAlignment.center,
        runSpacing: 10,
        spacing: 16,
        children: [
          Text(
            '${entry.language.label} · ${localSize(entry.archiveBytes)}',
            style: Fonts.sans(size: 14, color: p.text),
          ),
          _state(context, p),
        ],
      ),
    );
  }
}
