//! État du champ « ce que tu dois savoir de moi » : texte enregistré, brouillon, enregistrement et erreur.

import 'package:flutter/foundation.dart';

import '../engine/ai_about_admin.dart';
import '../engine/ai_admin.dart';
import '../i18n/engine_errors.dart';

/// Ce que montrent l'onglet Mémoire et l'étape d'accueil ; rien ne part tant que l'utilisateur n'enregistre pas.
class AboutMeState extends ChangeNotifier {
  final AiAdmin admin;
  AboutMeState(this.admin);

  String saved = '';
  String draft = '';
  bool loaded = false;
  bool saving = false;
  bool justSaved = false;
  String? error;
  bool _disposed = false;

  /// Dit si le brouillon diffère du texte enregistré.
  bool get dirty => draft.trim() != saved;

  /// Dit si le brouillon respecte la limite du moteur.
  bool get valid => draft.trim().runes.length <= aboutMaxChars;

  /// Lit le texte enregistré ; il devient aussi le brouillon.
  Future<void> load() async {
    try {
      saved = await admin.aboutMe();
      draft = saved;
      error = null;
    } on Object catch (e) {
      error = engineErrorText(e);
    }
    loaded = true;
    _changed();
  }

  /// Retient le brouillon en cours de frappe, sans l'enregistrer.
  void edit(String text) {
    draft = text;
    justSaved = false;
    notifyListeners();
  }

  /// Enregistre le brouillon ; en cas d'échec le texte enregistré reste.
  Future<bool> save() async {
    if (!dirty || !valid) return false;
    saving = true;
    error = null;
    _changed();
    try {
      saved = await admin.setAboutMe(draft);
      draft = saved;
      justSaved = true;
    } on Object catch (e) {
      error = engineErrorText(e);
    }
    saving = false;
    _changed();
    return error == null;
  }

  void _changed() {
    if (!_disposed) notifyListeners();
  }

  @override
  void dispose() {
    _disposed = true;
    super.dispose();
  }
}
