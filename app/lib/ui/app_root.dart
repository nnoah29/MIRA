import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../state/store.dart';
import 'shell.dart';
import 'welcome_page.dart';

/// Racine de l'écran : l'accueil du premier lancement tant qu'il est ouvert, sinon la messagerie.
class AppRoot extends StatelessWidget {
  const AppRoot({super.key});

  @override
  Widget build(BuildContext context) {
    final welcome = context.select<MailStore, bool>((s) => s.welcomeOpen);
    return welcome ? const WelcomePage() : const Shell();
  }
}
