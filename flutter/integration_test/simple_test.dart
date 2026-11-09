import 'package:flutter_test/flutter_test.dart';
import 'package:matrix_client/main.dart';
import 'package:matrix_client/src/rust/frb_generated.dart';
import 'package:integration_test/integration_test.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => await RustLib.init());
  testWidgets('renders the login form', (WidgetTester tester) async {
    await tester.pumpWidget(const MatrixClientApp());
    expect(find.text('Log in'), findsOneWidget);
  });
}
