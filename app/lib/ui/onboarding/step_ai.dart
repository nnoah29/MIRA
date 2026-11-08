import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../engine/ai_models.dart';
import '../../i18n/i18n.dart';
import '../../state/ai_settings_state.dart';
import '../../state/store.dart';
import '../../theme/app_theme.dart';
import '../ai_browser_login.dart';
import '../ai_connection_card.dart';
import '../widgets.dart';
import 'onboarding_controller.dart';
import 'step_layout.dart';

/// Page 5 (optionnelle) : explique l'IA et propose la connexion par navigateur d'un service qui la permet.
class AiStep extends StatefulWidget {
  final OnboardingController controller;
  const AiStep({super.key, required this.controller});

  @override
  State<AiStep> createState() => _AiStepState();
}

class _AiStepState extends State<AiStep> {
  late final AiSettingsState _state = AiSettingsState(
    context.read<MailStore>().aiAdmin!,
  );

  @override
  void initState() {
    super.initState();
    _state.addListener(_refresh);
    _state.load();
  }

  @override
  void dispose() {
    _state.removeListener(_refresh);
    _state.dispose();
    super.dispose();
  }

  void _refresh() {
    if (mounted) setState(() {});
  }

  AiProviderInfo? get _provider {
    for (final p in _state.providers) {
      if (p.keyLogin) return p;
    }
    return null;
  }

  Future<void> _login(AiProviderInfo provider) async {
    if (_state.connectionsOf(provider.id).isEmpty) {
      await _state.addConnection(provider.id, label: provider.name);
    }
    final made = _state.connectionsOf(provider.id);
    if (made.isEmpty) return;
    final id = made.first.id;
    await _state.loginByBrowser(id, openLoginPage);
    final done = _state.connections.any((c) => c.id == id && c.hasKey);
    if (mounted && done) widget.controller.next();
  }

  Widget _waiting(Pal p) => Row(
    children: [
      Text(
        t('En attente du navigateur…'),
        style: Fonts.sans(size: 14, color: p.muted),
      ),
      const SizedBox(width: 12),
      Btn(
        key: const ValueKey('onb-ai-cancel'),
        label: t('Annuler'),
        onTap: () => _state.cancelLogin(_state.logins.keys.first),
      ),
    ],
  );

  Widget _actions(Pal p, AiProviderInfo? provider) {
    if (provider == null) {
      return Text(
        t('Tu pourras brancher un service d’IA dans Réglages › IA.'),
        style: Fonts.sans(size: 15, color: p.dim),
      );
    }
    if (_state.logins.isNotEmpty) return _waiting(p);
    return Align(
      alignment: Alignment.centerLeft,
      child: Btn(
        key: const ValueKey('onb-ai-login'),
        label: t('Se connecter avec {0}', [aiProviderTitle(provider)]),
        go: true,
        onTap: _state.busy.isNotEmpty ? null : () => _login(provider),
      ),
    );
  }

  Widget _notes(Pal p) {
    final link = _state.loginLinks.values.firstOrNull;
    final error = _state.error;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (link != null)
          Padding(
            padding: const EdgeInsets.only(top: 12),
            child: SelectableText(
              '${t('Le navigateur ne s’est pas ouvert : copie ce lien dans ton navigateur.')}\n$link',
              key: const ValueKey('onb-ai-link'),
              style: Fonts.sans(size: 13, color: p.dim),
            ),
          ),
        if (error != null)
          Padding(
            padding: const EdgeInsets.only(top: 12),
            child: Text(
              error,
              key: const ValueKey('onb-ai-error'),
              style: Fonts.sans(size: 13.5, color: p.accent),
            ),
          ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return StepLayout(
      title: t('ASSISTANT D’IA'),
      lead: t(
        'Ton texte n’est envoyé qu’avec ton accord, les données sensibles sont masquées avant l’envoi. Ta clé reste sur cet appareil.',
      ),
      children: [
        _actions(p, _provider),
        _notes(p),
        const SizedBox(height: 20),
        Btn(
          key: const ValueKey('onb-ai-later'),
          label: t('Plus tard (Réglages › IA)'),
          onTap: widget.controller.next,
        ),
      ],
    );
  }
}
