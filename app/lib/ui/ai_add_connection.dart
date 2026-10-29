//! Ajout d'une connexion d'IA : un bouton par fournisseur, une adresse à saisir pour le fournisseur libre.

import 'package:flutter/material.dart';

import '../engine/ai_models.dart';
import '../i18n/i18n.dart';
import '../state/ai_settings_state.dart';
import '../theme/app_theme.dart';
import 'ai_connection_card.dart';
import 'widgets.dart';

/// Liste des fournisseurs du moteur, chacun avec son bouton « Ajouter ».
class AiAddConnection extends StatefulWidget {
  final AiSettingsState state;
  const AiAddConnection({super.key, required this.state});

  @override
  State<AiAddConnection> createState() => _AiAddConnectionState();
}

class _AiAddConnectionState extends State<AiAddConnection> {
  AiProviderInfo? freeform;
  final label = TextEditingController();
  final url = TextEditingController();

  @override
  void dispose() {
    label.dispose();
    url.dispose();
    super.dispose();
  }

  Future<void> _add(AiProviderInfo p) async {
    if (p.editableUrl) return setState(() => freeform = p);
    await widget.state.addConnection(p.id, label: p.name);
  }

  Future<void> _addFree(AiProviderInfo p) async {
    final name = label.text.trim().isEmpty ? aiProviderTitle(p) : label.text;
    await widget.state.addConnection(
      p.id,
      label: name,
      baseUrl: url.text.trim(),
    );
    if (!mounted || widget.state.error != null) return;
    label.clear();
    url.clear();
    setState(() => freeform = null);
  }

  Widget _row(AiProviderInfo p, Pal p2) => Container(
    key: ValueKey('ai-provider-${p.id}'),
    padding: const EdgeInsets.symmetric(vertical: 10),
    decoration: BoxDecoration(
      border: Border(bottom: BorderSide(color: p2.lineSoft)),
    ),
    child: Row(
      children: [
        Expanded(
          child: Text(
            aiProviderTitle(p),
            style: Fonts.sans(size: 15, color: p2.text),
          ),
        ),
        Btn(
          key: ValueKey('ai-add-${p.id}'),
          icon: 'plus',
          label: t('Ajouter'),
          onTap: widget.state.busy.contains('new') ? null : () => _add(p),
        ),
      ],
    ),
  );

  Widget _form(AiProviderInfo p, Pal p2) => Padding(
    padding: const EdgeInsets.only(top: 12),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        TextField(
          key: const ValueKey('ai-new-label'),
          controller: label,
          decoration: InputDecoration(
            isDense: true,
            labelText: t('Nom de la connexion'),
          ),
        ),
        const SizedBox(height: 10),
        TextField(
          key: const ValueKey('ai-new-url'),
          controller: url,
          decoration: InputDecoration(
            isDense: true,
            labelText: t('Adresse du service'),
            hintText: 'https://…', // i18n:ignore
          ),
        ),
        const SizedBox(height: 12),
        Row(
          children: [
            Btn(
              label: t('Annuler'),
              onTap: () => setState(() => freeform = null),
            ),
            const SizedBox(width: 10),
            Btn(
              key: const ValueKey('ai-new-save'),
              label: t('Ajouter'),
              go: true,
              onTap: () => _addFree(p),
            ),
          ],
        ),
      ],
    ),
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final free = freeform;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(
          t('Ajouter une connexion'),
          style: Fonts.sans(size: 14, w: FontWeight.w500, color: p.text),
        ),
        const SizedBox(height: 6),
        for (final provider in widget.state.providers) _row(provider, p),
        if (free != null) _form(free, p),
      ],
    );
  }
}
