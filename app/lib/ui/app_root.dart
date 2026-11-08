import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../engine/remote/resident.dart';
import '../platform_info.dart';

import '../state/store.dart';
import 'resident_prompt.dart';
import 'shell.dart';
import 'welcome_page.dart';

/// Racine de l'écran : l'accueil du premier lancement tant qu'il est ouvert, sinon la messagerie.
class AppRoot extends StatefulWidget {
  const AppRoot({super.key});

  @override
  State<AppRoot> createState() => _AppRootState();
}

class _AppRootState extends State<AppRoot> {
  bool _asked = false;

  /// Pose la question du mode résident une fois l'accueil passé, sous Linux de bureau et Windows.
  void _askResident() {
    if (_asked || !platformHasDaemon) return;
    _asked = true;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      final store = context.read<MailStore>();
      askResidentOnce(
        context: context,
        prefs: store.prefs,
        controller: ResidentController.forPlatform(),
      );
    });
  }

  @override
  Widget build(BuildContext context) {
    final welcome = context.select<MailStore, bool>((s) => s.welcomeOpen);
    if (!welcome) _askResident();
    return welcome ? const WelcomePage() : const Shell();
  }
}
