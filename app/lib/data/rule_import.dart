/// Sort d'une règle du fichier importé : importée, remplacée, déjà présente ou refusée.
class RuleImportEntry {
  final int position;
  final String name;
  final String outcome;
  final String? reason;
  final String? code;
  const RuleImportEntry(
    this.position,
    this.name,
    this.outcome, {
    this.reason,
    this.code,
  });

  /// Vrai quand la règle n'a pas pu être importée.
  bool get refused => outcome == 'refused';

  factory RuleImportEntry.fromJson(Map<String, dynamic> json) {
    final outcome = (json['outcome'] as Map).cast<String, dynamic>();
    final reason = (outcome['reason'] as Map?)?.cast<String, dynamic>();
    return RuleImportEntry(
      json['position'] as int,
      json['name'] as String,
      outcome['type'] as String,
      reason: reason?['type'] as String?,
      code: reason?['code'] as String?,
    );
  }
}

/// Rapport d'import : une ligne par règle du fichier, dans l'ordre du fichier.
class RuleImportReport {
  final List<RuleImportEntry> entries;
  const RuleImportReport(this.entries);

  int _count(String outcome) =>
      entries.where((e) => e.outcome == outcome).length;

  int get imported => _count('imported');
  int get replaced => _count('replaced');
  int get alreadyPresent => _count('already_present');
  int get refused => _count('refused');

  factory RuleImportReport.fromJson(Map<String, dynamic> json) =>
      RuleImportReport([
        for (final e in json['entries'] as List)
          RuleImportEntry.fromJson((e as Map).cast<String, dynamic>()),
      ]);
}

/// Texte YAML exporté et noms des règles laissées de côté parce qu'une de leurs cibles a disparu.
class RulesExport {
  final String yaml;
  final List<String> skipped;
  const RulesExport(this.yaml, this.skipped);

  factory RulesExport.fromJson(Map<String, dynamic> json) => RulesExport(
    json['yaml'] as String,
    [for (final n in (json['skipped'] as List? ?? const [])) n as String],
  );
}
