import 'package:flutter/material.dart';
import 'src/controllers/platform_controller.dart';
import 'src/screens/shell_screen.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  final controller = PlatformController();
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
