//! Carte d'une connexion d'IA : clé en écriture seule, modèle, essai, connexion par défaut, suppression.

import 'package:flutter/material.dart';

import '../engine/ai_models.dart';
import '../i18n/i18n.dart';
import '../state/ai_settings_state.dart';
import '../theme/app_theme.dart';
import 'ai_model_picker.dart';
import 'ai_privacy_section.dart';
import 'account_form_widgets.dart';
import 'widgets.dart';

/// Nom du fournisseur tel que l'app l'affiche (le fournisseur libre se présente comme « Autre »).
String aiProviderTitle(AiProviderInfo p) =>
    p.editableUrl ? t('Autre (compatible OpenAI)') : p.name;

/// Carte d'une connexion ; toutes les actions passent par [state].
class AiConnectionCard extends StatefulWidget {
  final AiSettingsState state;
  final AiConnectionInfo connection;
  const AiConnectionCard({
    super.key,
    required this.state,
    required this.connection,
  });

  @override
  State<AiConnectionCard> createState() => _AiConnectionCardState();
}

class _AiConnectionCardState extends State<AiConnectionCard> {
  final keyCtl = TextEditingController();
  late final TextEditingController urlCtl = TextEditingController(
    text: widget.connection.baseUrl ?? '',
  );

  AiConnectionInfo get c => widget.connection;
  String get id => c.id;

  @override
  void dispose() {
    keyCtl.dispose();
    urlCtl.dispose();
    super.dispose();
  }

  Future<void> _saveKey() async {
    final typed = keyCtl.text;
    if (typed.trim().isEmpty) return;
    if (await widget.state.saveKey(id, typed)) keyCtl.clear();
  }

  Future<void> _testNow() async {
    await _saveKey();
    if (keyCtl.text.trim().isNotEmpty) return;
    await widget.state.testConnection(id);
  }

  Future<void> _confirmDelete() async {
    final p = context.pal;
    final ok = await showDialog<bool>(
      context: context,
      builder: (d) => Dialog(
        backgroundColor: p.bgSide,
        shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 420),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  t('Supprimer la connexion « {0} » ?', [c.label]),
                  style: Fonts.sans(
                    size: 16,
                    w: FontWeight.w600,
                    color: p.text,
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  t('Sa clé d’API est effacée de cet appareil.'),
                  style: Fonts.sans(size: 13.5, color: p.dim),
                ),
                const SizedBox(height: 20),
                Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    Btn(
                      label: t('Annuler'),
                      onTap: () => Navigator.pop(d, false),
                    ),
                    const SizedBox(width: 10),
                    Btn(
                      key: const ValueKey('confirm-delete-ai'),
                      label: t('Supprimer'),
                      go: true,
                      onTap: () => Navigator.pop(d, true),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      ),
    );
    if (ok == true) await widget.state.remove(id);
  }

  bool _sameTitle(AiProviderInfo? provider) =>
      provider == null || aiProviderTitle(provider) == c.label;

  Widget _title(Pal p, String title, bool own) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          c.label,
          style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
        ),
        if (title != c.label) ...[
          const SizedBox(height: 2),
          Text(title, style: Fonts.sans(size: 12.5, color: p.dim)),
        ],
      ],
    );
  }

  Widget _controls(Pal p) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      Hov(
        key: ValueKey('ai-default-$id'),
        onTap: c.isDefault ? null : () => widget.state.setDefault(id),
        builder: (_, _) => Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              c.isDefault ? Icons.radio_button_checked : Icons.radio_button_off,
              size: 18,
              color: c.isDefault ? p.accent : p.dim,
            ),
            const SizedBox(width: 6),
            Text(t('Par défaut'), style: Fonts.sans(size: 13, color: p.muted)),
          ],
        ),
      ),
      const SizedBox(width: 10),
      IBtn(
        'trash',
        key: ValueKey('ai-delete-$id'),
        tip: t('Supprimer'),
        onTap: _confirmDelete,
      ),
    ],
  );

  Widget _address(Pal p) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      FormField2(
        fieldKey: ValueKey('ai-url-$id'),
        controller: urlCtl,
        label: t('Adresse du service'),
        hint: 'https://…', // i18n:ignore
      ),
      const SizedBox(height: 10),
      Btn(
        key: ValueKey('ai-url-save-$id'),
        label: t('Enregistrer'),
        onTap: () => widget.state.saveAddress(c, c.label, urlCtl.text.trim()),
      ),
    ],
  );

  Widget _saved(Pal p) => Padding(
    padding: const EdgeInsets.only(left: 14),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(Icons.check, size: 14, color: p.accent),
        const SizedBox(width: 6),
        Text(
          t('Clé enregistrée'),
          key: ValueKey('ai-key-saved-$id'),
          style: Fonts.sans(size: 13, color: p.muted),
        ),
      ],
    ),
  );

  Widget _key(Pal p) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      FormField2(
        fieldKey: ValueKey('ai-key-field-$id'),
        controller: keyCtl,
        secret: true,
        onSubmitted: (_) => _saveKey(),
        label: c.hasKey ? t('Remplacer la clé d’API') : t('Clé d’API'),
      ),
      const SizedBox(height: 10),
      Row(
        children: [
          Btn(
            key: ValueKey('ai-key-save-$id'),
            label: t('Enregistrer la clé'),
            onTap: _saveKey,
          ),
          if (c.hasKey) ...[
            const SizedBox(width: 8),
            Btn(
              key: ValueKey('ai-key-clear-$id'),
              label: t('Effacer la clé'),
              onTap: () => widget.state.clearKey(id),
            ),
            _saved(p),
          ],
        ],
      ),
    ],
  );

  Widget _testButton(bool busy) => Btn(
    key: ValueKey('ai-test-$id'),
    label: t('Tester'),
    onTap: busy ? null : _testNow,
  );

  Widget _outcome(Pal p) {
    final outcome = widget.state.tests[id];
    if (outcome == null) return const SizedBox.shrink();
    return Padding(
      padding: const EdgeInsets.only(top: 8),
      child: Text(
        outcome.ok ? t('Connexion réussie.') : outcome.error ?? '',
        key: ValueKey('ai-test-result-$id'),
        style: Fonts.sans(size: 13.5, color: outcome.ok ? p.muted : p.accent),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final state = widget.state;
    final provider = state.providerOf(c);
    final busy = state.busy.contains(id);
    return Container(
      key: ValueKey('ai-card-$id'),
      margin: const EdgeInsets.only(top: 14),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: p.surface,
        border: Border.all(color: p.line),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (provider?.editableUrl == true || !_sameTitle(provider)) ...[
            _title(
              p,
              provider == null ? c.provider : aiProviderTitle(provider),
              true,
            ),
            const SizedBox(height: 16),
          ],
          if (provider?.editableUrl == true) ...[
            _address(p),
            const SizedBox(height: 16),
          ],
          _key(p),
          const SizedBox(height: 16),
          AiModelPicker(state: state, connection: c),
          const SizedBox(height: 16),
          AiPrivacySection(state: state, connection: c),
          const SizedBox(height: 16),
          Row(children: [_testButton(busy), const Spacer(), _controls(p)]),
          _outcome(p),
        ],
      ),
    );
  }
}
