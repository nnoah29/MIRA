import '../data/models.dart';

/// Deux chiffres au moins, pour les dates et heures.
String _two(int n) => n.toString().padLeft(2, '0');

/// Destinataire principal pour les variables `contact.*` ; vide si l'on ne sait rien de lui.
Map<String, String> _contactOf(List<Person> to) {
  if (to.isEmpty) return const {};
  final name = to.first.name.trim();
  final named = name.isNotEmpty && !name.contains('@');
  return {
    if (named) 'name': name,
    if (named) 'first_name': name.split(RegExp(r'\s+')).first,
    if (to.first.email.isNotEmpty) 'email': to.first.email,
  };
}

/// Valeurs que le moteur utilise pour remplacer les variables ; une valeur vide n'est pas envoyée.
Map<String, dynamic> buildTemplateContext({
  required List<Person> to,
  required String subject,
  required String fromEmail,
  String? fromName,
  required DateTime now,
}) {
  final contact = _contactOf(to);
  final name = fromName?.trim() ?? '';
  return {
    if (contact.isNotEmpty) 'contact': contact,
    if (name.isNotEmpty) 'account_name': name,
    if (fromEmail.isNotEmpty) 'account_email': fromEmail,
    if (subject.trim().isNotEmpty) 'subject': subject.trim(),
    'date': '${_two(now.day)}/${_two(now.month)}/${now.year}',
    'time': '${_two(now.hour)}:${_two(now.minute)}',
  };
}
