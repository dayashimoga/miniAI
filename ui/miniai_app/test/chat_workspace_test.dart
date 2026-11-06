import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniai_app/src/controllers/platform_controller.dart';
import 'package:miniai_app/src/models/system_contracts.dart';
import 'package:miniai_app/src/screens/chat_workspace_screen.dart';
import 'package:miniai_app/src/screens/shell_screen.dart';
import 'package:miniai_app/src/services/settings_store.dart';

void main() {
  group('Subphase 3.2.2 Chat Session & PlatformController Unit Tests', () {
    late PlatformController controller;

    setUp(() {
      controller = PlatformController(
        settingsStore: InMemorySettingsStore(),
        initialHardware: const HardwareProfile(
          osFamily: 'windows',
          arch: 'x86_64',
          logicalCpuCores: 8,
          totalMemoryBytes: 16 * 1024 * 1024 * 1024,
          availableMemoryBytes: 8 * 1024 * 1024 * 1024,
          detectedBackends: [AcceleratorBackend.cpu],
        ),
      );
    });

    test('Initializes with default session and welcome assistant message', () {
      expect(controller.chatSessions.length, equals(1));
      final defaultSession = controller.chatSessions.first;
      expect(defaultSession.title, equals('MiniAI Offline Chat'));
      expect(controller.activeSessionId, equals(defaultSession.id));
      expect(controller.currentMessages.length, equals(1));
      expect(controller.currentMessages.first.role, equals(ChatMessageRole.assistant));
      expect(controller.currentMessages.first.content, contains('Welcome to MiniAI'));
    });

    test('Creates, selects, renames, and deletes chat sessions', () {
      final session2 = controller.createChatSession(title: 'Research Session');
      expect(controller.chatSessions.length, equals(2));
      expect(controller.activeSessionId, equals(session2.id));
      expect(controller.currentMessages.isEmpty, isTrue);

      controller.renameChatSession(session2.id, 'Deep Learning Exploration');
      expect(controller.activeSession?.title, equals('Deep Learning Exploration'));

      // Switch back to initial session
      final firstSession = controller.chatSessions.last;
      controller.selectChatSession(firstSession.id);
      expect(controller.activeSessionId, equals(firstSession.id));
      expect(controller.currentMessages.length, equals(1));

      // Delete session2
      controller.deleteChatSession(session2.id);
      expect(controller.chatSessions.length, equals(1));
      expect(controller.chatSessions.any((s) => s.id == session2.id), isFalse);
    });

    test('Model tuning hyperparameters setter clamps within valid range', () {
      controller.setSelectedChatModel('llama-3.2-1b-instruct-q4');
      expect(controller.selectedChatModelId, equals('llama-3.2-1b-instruct-q4'));

      controller.setChatTemperature(0.85);
      expect(controller.chatTemperature, closeTo(0.85, 0.001));
      controller.setChatTemperature(-0.5);
      expect(controller.chatTemperature, equals(0.0));
      controller.setChatTemperature(3.5);
      expect(controller.chatTemperature, equals(2.0));

      controller.setChatMaxTokens(512);
      expect(controller.chatMaxTokens, equals(512));
      controller.setChatMaxTokens(5);
      expect(controller.chatMaxTokens, equals(16));
      controller.setChatMaxTokens(9999);
      expect(controller.chatMaxTokens, equals(4096));
    });

    test('Sends message and generates tokens synchronously without delay in test mode', () async {
      await controller.sendMessage('Explain offline inference', delayTokens: false);
      expect(controller.currentMessages.length, equals(3)); // welcome + user + assistant
      final userMsg = controller.currentMessages[1];
      final assistantMsg = controller.currentMessages[2];

      expect(userMsg.role, equals(ChatMessageRole.user));
      expect(userMsg.content, equals('Explain offline inference'));
      expect(userMsg.tokenCount, greaterThan(0));

      expect(assistantMsg.role, equals(ChatMessageRole.assistant));
      expect(assistantMsg.content, contains('Executing inference with llama-3.2-1b-instruct-q4'));
      expect(assistantMsg.tokenCount, greaterThan(10));
      expect(controller.isGenerating, isFalse);
    });

    test('Edits and deletes message within active session', () async {
      await controller.sendMessage('Original message', delayTokens: false);
      final userMsg = controller.currentMessages[1];

      controller.editMessage(userMsg.id, 'Corrected query statement');
      expect(controller.currentMessages[1].content, equals('Corrected query statement'));

      controller.deleteMessage(userMsg.id);
      expect(controller.currentMessages.any((m) => m.id == userMsg.id), isFalse);
    });

    test('Exports session to formatted Markdown and JSON', () async {
      await controller.sendMessage('Test prompt for export', delayTokens: false);
      final activeId = controller.activeSessionId!;

      final md = controller.exportSessionMarkdown(activeId);
      expect(md, contains('# MiniAI Chat Export'));
      expect(md, contains('### 👤 User'));
      expect(md, contains('Test prompt for export'));
      expect(md, contains('### 🤖 Assistant'));

      final jsonStr = controller.exportSessionJson(activeId);
      expect(jsonStr, contains('"session"'));
      expect(jsonStr, contains('"messages"'));
      expect(jsonStr, contains('Test prompt for export'));
    });

    test('Memory admission failure produces polite diagnostic message', () async {
      // Create controller with minimal memory
      final restrictedController = PlatformController(
        initialHardware: const HardwareProfile(
          osFamily: 'windows',
          arch: 'x86_64',
          logicalCpuCores: 4,
          totalMemoryBytes: 1024 * 1024 * 100, // 100 MB total
          availableMemoryBytes: 1024 * 1024 * 50, // 50 MB available
          detectedBackends: [AcceleratorBackend.cpu],
        ),
      );

      // llama-3.2-1b requires ~2GB RAM, which exceeds 50MB
      await restrictedController.sendMessage('Test query', delayTokens: false);
      final lastMsg = restrictedController.currentMessages.last;
      expect(lastMsg.content, contains('Memory Admission Error'));
    });
  });

  group('ChatWorkspaceScreen Widget & Streaming Interaction Tests', () {
    late PlatformController controller;

    setUp(() {
      controller = PlatformController(
        settingsStore: InMemorySettingsStore(),
        initialHardware: const HardwareProfile(
          osFamily: 'windows',
          arch: 'x86_64',
          logicalCpuCores: 8,
          totalMemoryBytes: 16 * 1024 * 1024 * 1024,
          availableMemoryBytes: 8 * 1024 * 1024 * 1024,
          detectedBackends: [AcceleratorBackend.cpu],
        ),
      );
    });

    testWidgets('Renders header toolbar, model selector, tuning, and export controls',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1200, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: ChatWorkspaceScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('MiniAI Offline Chat'), findsWidgets);
      expect(find.textContaining('Offline-first • CPU'), findsOneWidget);
      expect(find.byKey(const Key('chat_model_selector')), findsOneWidget);
      expect(find.byKey(const Key('tuning_settings_button')), findsOneWidget);
      expect(find.byKey(const Key('export_chat_button')), findsOneWidget);
      expect(find.byKey(const Key('chat_input_field')), findsOneWidget);
      expect(find.byKey(const Key('send_message_button')), findsOneWidget);
    });

    testWidgets('Session sidebar supports creating, renaming, and deleting conversations',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1200, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: ChatWorkspaceScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      // Tap New Chat button
      expect(find.byKey(const Key('new_chat_button')), findsOneWidget);
      await tester.tap(find.byKey(const Key('new_chat_button')));
      await tester.pumpAndSettle();

      expect(controller.chatSessions.length, equals(2));
      final newSessionId = controller.activeSessionId!;

      // Rename new session
      final renameKey = Key('rename_session_$newSessionId');
      expect(find.byKey(renameKey), findsOneWidget);
      await tester.tap(find.byKey(renameKey));
      await tester.pumpAndSettle();

      // Enter new title in dialog
      expect(find.byKey(const Key('rename_session_input')), findsOneWidget);
      await tester.enterText(
        find.byKey(const Key('rename_session_input')),
        'Autonomous Agents Discussion',
      );
      await tester.tap(find.byKey(const Key('confirm_rename_session_button')));
      await tester.pumpAndSettle();

      expect(controller.activeSession?.title, equals('Autonomous Agents Discussion'));
      expect(find.text('Autonomous Agents Discussion'), findsWidgets);

      // Delete session
      final deleteKey = Key('delete_session_$newSessionId');
      await tester.tap(find.byKey(deleteKey));
      await tester.pumpAndSettle();

      expect(controller.chatSessions.length, equals(1));
    });

    testWidgets('Sends message and displays user bubble and real-time generation',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1200, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: ChatWorkspaceScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      // Enter prompt in input field
      await tester.enterText(
        find.byKey(const Key('chat_input_field')),
        'What are the core guarantees of MiniAI?',
      );
      await tester.tap(find.byKey(const Key('send_message_button')));

      // Pump frame to show user message and streaming state
      await tester.pump(const Duration(milliseconds: 10));

      expect(find.text('What are the core guarantees of MiniAI?'), findsOneWidget);
      expect(find.byKey(const Key('generation_progress_card')), findsOneWidget);
      expect(find.byKey(const Key('cancel_generation_button')), findsOneWidget);

      // Tap cancel generation button
      await tester.tap(find.byKey(const Key('cancel_generation_button')));
      await tester.pumpAndSettle();

      expect(controller.isGenerating, isFalse);
      expect(find.byKey(const Key('generation_progress_card')), findsNothing);
    });

    testWidgets('Edits user message and opens tuning hyperparameters dialog',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1200, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      // Prepopulate with a user message
      await controller.sendMessage('Draft prompt question', delayTokens: false);

      await tester.pumpWidget(
        MaterialApp(
          home: ChatWorkspaceScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      final userMsg = controller.currentMessages[1];
      final editButtonKey = Key('edit_message_${userMsg.id}');
      expect(find.byKey(editButtonKey), findsOneWidget);

      // Open edit dialog
      await tester.tap(find.byKey(editButtonKey));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('edit_message_input')), findsOneWidget);
      await tester.enterText(
        find.byKey(const Key('edit_message_input')),
        'Updated prompt question with details',
      );
      await tester.tap(find.byKey(const Key('confirm_edit_message_button')));
      await tester.pumpAndSettle();

      expect(find.text('Updated prompt question with details'), findsOneWidget);

      // Open tuning dialog
      await tester.tap(find.byKey(const Key('tuning_settings_button')));
      await tester.pumpAndSettle();

      expect(find.text('Inference Hyperparameters'), findsOneWidget);
      expect(find.byKey(const Key('temperature_slider')), findsOneWidget);
      expect(find.byKey(const Key('max_tokens_slider')), findsOneWidget);

      await tester.tap(find.text('Done'));
      await tester.pumpAndSettle();
      expect(find.text('Inference Hyperparameters'), findsNothing);
    });

    testWidgets('ShellScreen navigates to Chat tab (tab 5) via desktop and mobile navigation',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1024, 768);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: ShellScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      // Navigate to Chat Tab via Desktop NavigationRail
      expect(find.byKey(const Key('nav_rail_chat')), findsOneWidget);
      await tester.tap(find.byKey(const Key('nav_rail_chat')));
      await tester.pumpAndSettle();

      expect(controller.selectedNavigationIndex, equals(5));
      expect(find.byKey(const Key('chat_composer_bar')), findsOneWidget);
      expect(find.byKey(const Key('chat_input_field')), findsOneWidget);

      // Switch to mobile viewport
      tester.view.physicalSize = const Size(480, 850);
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('mobile_navigation_bar')), findsOneWidget);
      expect(find.byKey(const Key('nav_bar_chat')), findsOneWidget);
      expect(find.byKey(const Key('toggle_sessions_button')), findsOneWidget);

      // Toggle sessions drawer on mobile
      await tester.tap(find.byKey(const Key('toggle_sessions_button')));
      await tester.pumpAndSettle();
      expect(find.text('Conversations'), findsOneWidget);
    });
  });
}
