import 'package:flutter_test/flutter_test.dart';
import 'package:mira/voice/voice_services.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  test('une erreur à l’ouverture des services de dictée ne bloque pas l’app : rien n’est proposé', () async {
    TestWidgetsFlutterBinding.ensureInitialized();
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();
    final services = await openVoiceServices(prefs, uiLocale: 'fr');
    expect(services, isNull);
  });
}
