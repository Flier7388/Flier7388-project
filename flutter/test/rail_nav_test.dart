// Headless widget tests for the sidebar rail — no server, no real display,
// just the widget tree. Exercises the exact wiring the project owner found
// broken by hand earlier (a space behaving like an openable room): here,
// the regression is "tapping a space must call onSelectSpace, not get
// mistaken for one of the plain nav tabs".

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:matrix_client/src/rust/api/matrix.dart';
import 'package:matrix_client/widgets/rail_nav.dart';

void main() {
  const space = SpaceSummary(
    roomId: '!space:example.org',
    displayName: 'Служебное пространство',
  );

  Widget buildRail({
    RailTab? activeTab = RailTab.home,
    String? activeSpaceId,
    required ValueChanged<RailTab> onSelect,
    required ValueChanged<SpaceSummary> onSelectSpace,
  }) {
    return MaterialApp(
      home: Scaffold(
        body: RailNav(
          activeTab: activeTab,
          spaces: [space],
          activeSpaceId: activeSpaceId,
          userId: '@test:example.org',
          onSelect: onSelect,
          onSelectSpace: onSelectSpace,
        ),
      ),
    );
  }

  testWidgets('renders a tab for every joined space', (tester) async {
    await tester.pumpWidget(
      buildRail(onSelect: (_) {}, onSelectSpace: (_) {}),
    );

    expect(find.byTooltip(space.displayName), findsOneWidget);
  });

  testWidgets('tapping a space calls onSelectSpace, not onSelect', (
    tester,
  ) async {
    SpaceSummary? selectedSpace;
    RailTab? selectedTab;

    await tester.pumpWidget(
      buildRail(
        onSelect: (tab) => selectedTab = tab,
        onSelectSpace: (s) => selectedSpace = s,
      ),
    );

    await tester.tap(find.byTooltip(space.displayName));
    await tester.pump();

    expect(selectedSpace?.roomId, space.roomId);
    expect(selectedTab, isNull);
  });

  testWidgets('tapping Home calls onSelect(RailTab.home)', (tester) async {
    RailTab? selectedTab;

    await tester.pumpWidget(
      buildRail(
        activeTab: null,
        onSelect: (tab) => selectedTab = tab,
        onSelectSpace: (_) {},
      ),
    );

    await tester.tap(find.byTooltip('Home'));
    await tester.pump();

    expect(selectedTab, RailTab.home);
  });
}
