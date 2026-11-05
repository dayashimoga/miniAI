import 'dart:convert';
import 'dart:io';
import '../models/system_contracts.dart';

/// Abstract storage interface for persisting application settings.
abstract class SettingsStore {
  Future<AppSettings> loadSettings();
  Future<void> saveSettings(AppSettings settings);
}

/// In-memory implementation of [SettingsStore], ideal for isolated unit and widget tests.
class InMemorySettingsStore implements SettingsStore {
  InMemorySettingsStore([AppSettings? initial])
      : _settings = initial ?? const AppSettings();

  AppSettings _settings;

  AppSettings get currentSettings => _settings;

  @override
  Future<AppSettings> loadSettings() async => _settings;

  @override
  Future<void> saveSettings(AppSettings settings) async {
    _settings = settings;
  }
}

/// Local file-backed implementation of [SettingsStore] ensuring offline persistence across app restarts.
class FileSettingsStore implements SettingsStore {
  FileSettingsStore({required this.filePath});

  final String filePath;
  Future<void> _writeQueue = Future.value();

  @override
  Future<AppSettings> loadSettings() async {
    // Wait for any queued writes to complete before reading
    await _writeQueue;
    try {
      final file = File(filePath);
      if (!await file.exists()) {
        return const AppSettings();
      }
      final contents = await file.readAsString();
      final map = jsonDecode(contents) as Map<String, dynamic>;
      return AppSettings.fromJson(map);
    } catch (_) {
      return const AppSettings();
    }
  }

  @override
  Future<void> saveSettings(AppSettings settings) {
    _writeQueue = _writeQueue.then((_) async {
      try {
        final file = File(filePath);
        if (!await file.parent.exists()) {
          await file.parent.create(recursive: true);
        }
        await file.writeAsString(jsonEncode(settings.toJson()), flush: true);
      } catch (_) {
        // Offline file write error handling - fail gracefully without crashing
      }
    });
    return _writeQueue;
  }
}
