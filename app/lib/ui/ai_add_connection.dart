//! Ajout d'une connexion pour un fournisseur : un bouton, et nom + adresse à saisir pour le fournisseur libre.

import 'package:flutter/material.dart';

import '../engine/ai_models.dart';
import '../i18n/i18n.dart';
import '../state/ai_settings_state.dart';
import '../theme/app_theme.dart';
import 'ai_connection_card.dart';
import 'widgets.dart';

/// Bouton « Ajouter » d'un fournisseur ; le fournisseur libre demande d'abord nom et adresse.
class AiAddConnection extends StatefulWidget {
  final AiSettingsState state;
  final AiProviderInfo provider;
  const AiAddConnection({
    super.key,
    required this.state,
    required this.provider,
  });

  @override
  State<AiAddConnection> createState() => _AiAddConnectionState();
}

class _AiAddConnectionState extends State<AiAddConnection> {
  bool freeform = false;
  final label = TextEditingController();
  final url = TextEditingController();

  @override
  void dispose() {
    label.dispose();
    url.dispose();
    super.dispose();
  }

  Future<void> _add(AiProviderInfo p) async {
    if (p.editableUrl) return setState(() => freeform = true);
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
    setState(() => freeform = false);
  }

  Widget _button(AiProviderInfo p) => Align(
    alignment: Alignment.centerLeft,
    child: Padding(
      padding: const EdgeInsets.only(top: 14),
      child: Btn(
        key: ValueKey('ai-add-${p.id}'),
        icon: 'plus',
        label: t('Ajouter'),
        onTap: widget.state.busy.contains('new') ? null : () => _add(p),
      ),
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
              onTap: () => setState(() => freeform = false),
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
    final p = widget.provider;
    return freeform ? _form(p, context.pal) : _button(p);
  }
}
