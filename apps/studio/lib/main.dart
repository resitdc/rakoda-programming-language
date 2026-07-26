import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:provider/provider.dart';
import 'package:window_manager/window_manager.dart';
import 'src/rust/frb_generated.dart';
import 'features/welcome/welcome_screen.dart';
import 'features/theme/theme_state.dart';
import 'features/editor/rpl_languages.dart';
import 'features/identity/identity_service.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  
  await IdentityService.init();
  
  try {
    await RustLib.init();
  } catch (e) {
    debugPrint('RustLib init failed (expected on web if WASM is missing): $e');
  }
  
  registerRplLanguages();

  if (!kIsWeb) {
    if (Platform.isWindows || Platform.isLinux || Platform.isMacOS) {
      await windowManager.ensureInitialized();
      WindowOptions windowOptions = const WindowOptions(
        size: Size(1200, 800),
        center: true,
        titleBarStyle: TitleBarStyle.normal,
      );
      windowManager.waitUntilReadyToShow(windowOptions, () async {
        await windowManager.show();
        await windowManager.focus();
        await windowManager.maximize();
      });
    }
  }

  runApp(
    ProviderScope(
      child: ChangeNotifierProvider(
        create: (_) => ThemeProvider(),
        child: const RplStudioApp(),
      ),
    ),
  );
}

class RplStudioApp extends StatelessWidget {
  const RplStudioApp({super.key});

  @override
  Widget build(BuildContext context) {
    final themeProvider = context.watch<ThemeProvider>();

    return MaterialApp(
      title: 'RPL Studio',
      debugShowCheckedModeBanner: false,
      theme: themeProvider.themeData,
      home: const WelcomeScreen(),
    );
  }
}
