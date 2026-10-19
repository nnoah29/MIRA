import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/search_filters.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/search_filters_bar.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  testWidgets('les pastilles posent et retirent les filtres', (tester) async {
    SharedPreferences.setMockInitialValues({});
    final store = MailStore(await SharedPreferences.getInstance());
    await tester.binding.setSurfaceSize(const Size(900, 600));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      ChangeNotifierProvider.value(
        value: store,
        child: MaterialApp(
          theme: buildTheme(true),
          home: const Scaffold(body: SearchFiltersBar(padding: 16)),
        ),
      ),
    );
    expect(find.byKey(const ValueKey('filter-reset')), findsNothing);
    await tester.tap(find.byKey(const ValueKey('filter-unread')));
    await tester.pump();
    expect(store.searchFilters.unread, isTrue);
    expect(find.byKey(const ValueKey('filter-reset')), findsOneWidget);
    await tester.enterText(find.byKey(const ValueKey('filter-from')), 'bob');
    await tester.pump();
    expect(store.searchFilters.from, 'bob');
    await tester.tap(find.byKey(const ValueKey('filter-reset')));
    await tester.pump();
    expect(store.searchFilters, const SearchFilters());
  });
}
