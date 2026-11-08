//! Connexion d'un service d'IA par le navigateur : bouton, attente avec annulation, lien à copier de secours.

import 'package:flutter/material.dart';

import '../engine/ai_models.dart';
import '../i18n/i18n.dart';
import '../platform_open.dart';
import '../state/ai_settings_state.dart';
import '../theme/app_theme.dart';
import 'ai_connection_card.dart';
import 'widgets.dart';

/// Ouvre l'adresse d'autorisation ; remplaçable dans les tests.
Future<bool> Function(Uri url) openLoginPage = openExternal;

/// Propose de se connecter avec son compte du service, sans clé à recopier ; le moteur range la clé reçue.
class AiBrowserLogin extends StatelessWidget {
  final AiSettingsState state;
  final AiConnectionInfo connection;
  final AiProviderInfo provider;
  const AiBrowserLogin({
    super.key,
    required this.state,
    required this.connection,
    required this.provider,
  });

  String get _id => connection.id;

  Widget _waiting(Pal p) => Row(
    children: [
      Text(
        t('En attente du navigateur…'),
        key: ValueKey('ai-login-waiting-$_id'),
        style: Fonts.sans(size: 13.5, color: p.muted),
      ),
      const SizedBox(width: 12),
      Btn(
        key: ValueKey('ai-login-cancel-$_id'),
        label: t('Annuler'),
        onTap: () => state.cancelLogin(_id),
      ),
    ],
  );

  Widget _button() => Align(
    alignment: Alignment.centerLeft,
    child: Btn(
      key: ValueKey('ai-login-$_id'),
      label: connection.hasKey
          ? t('Se reconnecter avec {0}', [aiProviderTitle(provider)])
          : t('Se connecter avec {0}', [aiProviderTitle(provider)]),
      go: !connection.hasKey,
      onTap: state.busy.contains(_id)
          ? null
          : () => state.loginByBrowser(_id, openLoginPage),
    ),
  );

  Widget _link(Pal p, Uri url) => Padding(
    padding: const EdgeInsets.only(top: 8),
    child: SelectableText(
      '${t('Le navigateur ne s’est pas ouvert : copie ce lien dans ton navigateur.')}\n$url',
      key: ValueKey('ai-login-link-$_id'),
      style: Fonts.sans(size: 12.5, color: p.dim),
    ),
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final link = state.loginLinks[_id];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        state.logins.containsKey(_id) ? _waiting(p) : _button(),
        if (link != null) _link(p, link),
        const SizedBox(height: 14),
        Text(
          t('Ou colle une clé d’API que tu as créée toi-même.'),
          style: Fonts.sans(size: 12.5, color: p.dim),
        ),
      ],
    );
  }
}
