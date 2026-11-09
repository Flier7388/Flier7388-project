// Smoke test: confirms the Rust bridge initializes and the login screen
// renders. See lib/main.dart and docs/ARCHITECTURE.md.

import 'package:flutter_test/flutter_test.dart';

import 'package:matrix_client/main.dart';
import 'package:matrix_client/src/rust/frb_generated.dart';

void main() {
  setUpAll(() async => await RustLib.init());

  testWidgets('renders the login form', (WidgetTester tester) async {
    await tester.pumpWidget(const MatrixClientApp());

    expect(find.text('Log in'), findsOneWidget);
  });
}
