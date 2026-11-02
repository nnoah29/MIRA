//! Règle de forme d'une adresse e-mail, identique à celle du moteur (`mira-vocabulary`).

/// Vrai si le moteur accepterait cette adresse comme destinataire.
bool isEngineAddress(String text) {
  final at = text.indexOf('@');
  if (at < 1) return false;
  final local = text.substring(0, at);
  final domain = text.substring(at + 1);
  return !_hasSpace(local) && _validDomain(domain);
}

bool _hasSpace(String s) => s.contains(RegExp(r'\s'));

bool _validDomain(String d) =>
    d.contains('.') &&
    !d.contains('@') &&
    !_hasSpace(d) &&
    !d.startsWith('.') &&
    !d.endsWith('.') &&
    !d.contains('..');
