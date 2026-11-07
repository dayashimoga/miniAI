import 'dart:io';
import 'package:flutter/material.dart';
import 'src/controllers/platform_controller.dart';
import 'src/screens/shell_screen.dart';
import 'src/services/settings_store.dart';

String resolveSettingsPath() {
  try {
    if (Platform.isWindows) {
      final appData = Platform.environment['APPDATA'] ?? '.';
      return '$appData/miniAI/settings.json';
    } else if (Platform.isMacOS) {
      final home = Platform.environment['HOME'] ?? '.';
      return '$home/Library/Application Support/miniAI/settings.json';
    } else if (Platform.isLinux) {
      final home = Platform.environment['HOME'] ?? '.';
      return '$home/.config/miniai/settings.json';
    }
  } catch (_) {
    // Web fallback
  }
  return 'settings.json';
}

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final settingsStore = FileSettingsStore(filePath: resolveSettingsPath());
  final controller = PlatformController(settingsStore: settingsStore);
  await controller.loadSettings();
  runApp(MiniAiApp(controller: controller));
}

class MiniAiApp extends StatelessWidget {
  const MiniAiApp({
    super.key,
    required this.controller,
  });

  final PlatformController controller;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'MiniAI',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF0F766E),
          brightness: Brightness.light,
        ),
        useMaterial3: true,
      ),
      darkTheme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF14B8A6),
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
      ),
      home: ShellScreen(controller: controller),
    );
  }
}
