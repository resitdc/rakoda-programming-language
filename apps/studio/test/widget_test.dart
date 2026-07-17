import 'package:flutter_test/flutter_test.dart';
import 'package:rpl_studio/main.dart';

void main() {
  testWidgets('App starts with WelcomeScreen', (WidgetTester tester) async {
    await tester.pumpWidget(const RplStudioApp());
    expect(find.text('RPL Studio'), findsOneWidget);
    expect(find.text('Buat Project'), findsOneWidget);
    expect(find.text('Buka Folder'), findsOneWidget);
  });
}
