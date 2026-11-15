//! Onglet « Mémoire » des paramètres : tout ce que l'assistant retient, séparé des connexions d'IA.

import 'package:flutter/material.dart';

import '../engine/ai_admin.dart';
import '../i18n/i18n.dart';
import '../state/about_me_state.dart';
import '../state/ai_memory_state.dart';
import '../theme/app_theme.dart';
import 'about_me_section.dart';
import 'ai_memory_section.dart';

/// Page de la mémoire : réglages, installation du modèle et souvenirs ; les connexions servent au mode « fournisseur ».
class MemorySettings extends StatefulWidget {
  final AiAdmin admin;
  const MemorySettings({super.key, required this.admin});

  @override
  State<MemorySettings> createState() => _MemorySettingsState();
}

class _MemorySettingsState extends State<MemorySettings> {
  late final AiMemoryState memory = AiMemoryState(widget.admin);
  late final AboutMeState about = AboutMeState(widget.admin);
  List<AiConnectionInfo> connections = const [];

  @override
  void initState() {
    super.initState();
    memory.load();
    about.load();
    _loadConnections();
  }

  /// Lit les connexions d'IA ; sans elles, seul le mode « sur cet appareil » reste proposé.
  Future<void> _loadConnections() async {
    try {
      final list = await widget.admin.connections();
      if (mounted) setState(() => connections = list);
    } on Object {
      return;
    }
  }

  @override
  void dispose() {
    memory.dispose();
    about.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Padding(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            t('Mémoire'),
            style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
          ),
          AboutMeSection(state: about),
          AiMemorySection(state: memory, connections: connections),
        ],
      ),
    );
  }
}
