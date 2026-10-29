//! Carte d'une connexion d'IA : clé en écriture seule, modèle, essai, connexion par défaut, suppression.

import 'package:flutter/material.dart';

import '../engine/ai_models.dart';
import '../i18n/i18n.dart';
import '../state/ai_settings_state.dart';
import '../theme/app_theme.dart';
import 'ai_model_picker.dart';
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

  Widget _head(Pal p, String title) => Row(
    children: [
      Expanded(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              c.label,
              style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
            ),
            const SizedBox(height: 2),
            Text(title, style: Fonts.sans(size: 12.5, color: p.dim)),
          ],
        ),
      ),
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

  Widget _address(Pal p) => Row(
    children: [
      Expanded(
        child: TextField(
          key: ValueKey('ai-url-$id'),
          controller: urlCtl,
          cursorColor: p.accent,
          style: Fonts.sans(size: 14, color: p.text),
          decoration: InputDecoration(
            isDense: true,
            labelText: t('Adresse du service'),
            hintText: 'https://…', // i18n:ignore
            border: OutlineInputBorder(borderSide: BorderSide(color: p.line)),
          ),
        ),
      ),
      const SizedBox(width: 8),
      Btn(
        key: ValueKey('ai-url-save-$id'),
        label: t('Enregistrer'),
        onTap: () => widget.state.saveAddress(c, c.label, urlCtl.text.trim()),
      ),
    ],
  );

  Widget _key(Pal p) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Row(
        children: [
          Expanded(
            child: TextField(
              key: ValueKey('ai-key-field-$id'),
              controller: keyCtl,
              obscureText: true,
              enableSuggestions: false,
              autocorrect: false,
              cursorColor: p.accent,
              style: Fonts.sans(size: 14, color: p.text),
              onSubmitted: (_) => _saveKey(),
              decoration: InputDecoration(
                isDense: true,
                labelText: c.hasKey
                    ? t('Remplacer la clé d’API')
                    : t('Clé d’API'),
                border: OutlineInputBorder(
                  borderSide: BorderSide(color: p.line),
                ),
              ),
            ),
          ),
          const SizedBox(width: 8),
          Btn(
            key: ValueKey('ai-key-save-$id'),
            label: t('Enregistrer la clé'),
            onTap: _saveKey,
          ),
        ],
      ),
      if (c.hasKey)
        Padding(
          padding: const EdgeInsets.only(top: 6),
          child: Row(
            children: [
              Icon(Icons.check, size: 14, color: p.accent),
              const SizedBox(width: 6),
              Text(
                t('Clé enregistrée'),
                key: ValueKey('ai-key-saved-$id'),
                style: Fonts.sans(size: 13, color: p.muted),
              ),
              const Spacer(),
              Btn(
                key: ValueKey('ai-key-clear-$id'),
                label: t('Effacer la clé'),
                onTap: () => widget.state.clearKey(id),
              ),
            ],
          ),
        ),
    ],
  );

  Widget _test(Pal p, bool busy) {
    final outcome = widget.state.tests[id];
    return Row(
      children: [
        Btn(
          key: ValueKey('ai-test-$id'),
          label: t('Tester'),
          onTap: busy ? null : _testNow,
        ),
        const SizedBox(width: 12),
        if (outcome != null)
          Expanded(
            child: Text(
              outcome.ok ? t('Connexion réussie.') : outcome.error ?? '',
              key: ValueKey('ai-test-result-$id'),
              style: Fonts.sans(
                size: 13.5,
                color: outcome.ok ? p.muted : p.accent,
              ),
            ),
          ),
      ],
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
        border: Border.all(color: c.isDefault ? p.accent : p.line),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _head(p, provider == null ? c.provider : aiProviderTitle(provider)),
          const SizedBox(height: 14),
          if (provider?.editableUrl == true) ...[
            _address(p),
            const SizedBox(height: 12),
          ],
          _key(p),
          const SizedBox(height: 12),
          AiModelPicker(
            connectionId: id,
            value: c.defaultModel,
            models: state.modelsOf(c),
            busy: busy,
            onChanged: (m) => state.setModel(c, m),
            onRefresh: () => state.loadModels(id, refresh: true),
          ),
          const SizedBox(height: 12),
          _test(p, busy),
        ],
      ),
    );
  }
}
