import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniai_app/main.dart';
import 'package:miniai_app/src/controllers/platform_controller.dart';
import 'package:miniai_app/src/models/system_contracts.dart';
import 'package:miniai_app/src/screens/dashboard_screen.dart';
import 'package:miniai_app/src/screens/event_inspector_view.dart';
import 'package:miniai_app/src/screens/settings_screen.dart';
import 'package:miniai_app/src/services/ipc_client.dart';
import 'package:miniai_app/src/services/settings_store.dart';

void main() {
  group('Subphase 1.3.1 Contract & SettingsStore Unit Tests', () {
    test('EventEnvelope and AppSettings JSON serialization round-trips', () {
      const event = EventEnvelope(
        sequence: 42,
        timestampUtc: '2026-11-05T12:00:00Z',
        eventType: 'privacy_policy_updated',
        payload: {'local_only_mode': true, 'reason': 'security_enforced'},
      );
      final eventJson = event.toJson();
      final eventDecoded = EventEnvelope.fromJson(eventJson);

      expect(eventDecoded.sequence, equals(42));
      expect(eventDecoded.timestampUtc, equals('2026-11-05T12:00:00Z'));
      expect(eventDecoded.eventType, equals('privacy_policy_updated'));
      expect(eventDecoded.payload['local_only_mode'], isTrue);

      const settings = AppSettings(
        localOnlyMode: false,
        forceCpuOnly: true,
        activeWorkspaceId: 'ml-research-01',
        ipcServerUrl: 'http://127.0.0.1:4140',
        sessionNonce: 'test-nonce-12345',
      );
      final settingsJson = settings.toJson();
      final settingsDecoded = AppSettings.fromJson(settingsJson);

      expect(settingsDecoded.localOnlyMode, isFalse);
      expect(settingsDecoded.forceCpuOnly, isTrue);
      expect(settingsDecoded.activeWorkspaceId, equals('ml-research-01'));
      expect(settingsDecoded.sessionNonce, equals('test-nonce-12345'));

      final modified = settings.copyWith(activeWorkspaceId: 'prod-02');
      expect(modified.activeWorkspaceId, equals('prod-02'));
      expect(modified.forceCpuOnly, isTrue);
    });

    test('FileSettingsStore persists and restores settings across restarts', () async {
      final tempDir = await Directory.systemTemp.createTemp('miniai_settings_test_');
      final settingsFile = '${tempDir.path}/settings.json';
      final store = FileSettingsStore(filePath: settingsFile);

      // Default when file does not exist
      final initial = await store.loadSettings();
      expect(initial.localOnlyMode, isTrue);
      expect(initial.activeWorkspaceId, equals('default'));

      // Save custom settings
      const updated = AppSettings(
        localOnlyMode: false,
        forceCpuOnly: true,
        activeWorkspaceId: 'custom-workspace-offline',
        ipcServerUrl: 'http://127.0.0.1:9999',
        sessionNonce: 'nonce-abc',
      );
      await store.saveSettings(updated);

      // Simulate app restart with a fresh store pointing to the same file
      final restartedStore = FileSettingsStore(filePath: settingsFile);
      final loaded = await restartedStore.loadSettings();

      expect(loaded.localOnlyMode, isFalse);
      expect(loaded.forceCpuOnly, isTrue);
      expect(loaded.activeWorkspaceId, equals('custom-workspace-offline'));
      expect(loaded.sessionNonce, equals('nonce-abc'));

      await tempDir.delete(recursive: true);
    });

    test('PlatformController records events, updates history, and applies filters', () {
      final store = InMemorySettingsStore();
      final controller = PlatformController(settingsStore: store);

      // Verify startup privacy event is recorded
      expect(controller.events.isNotEmpty, isTrue);
      expect(controller.events.first.eventType, equals('privacy_policy_updated'));

      // Change settings and verify event emissions
      controller.setForceCpuOnly(true);
      expect(controller.events.first.eventType, equals('hardware_changed'));

      controller.setActiveWorkspace('workspace-beta');
      expect(controller.events.first.eventType, equals('workspace_switched'));
      expect(controller.activeWorkspaceId, equals('workspace-beta'));

      controller.emitDiagnosticPing('Test ping');
      expect(controller.events.first.eventType, equals('diagnostic_alert'));

      // Filter events
      controller.setEventFilter('hardware_changed');
      expect(controller.filteredEvents.every((e) => e.eventType == 'hardware_changed'), isTrue);
      expect(controller.filteredEvents.length, equals(1));

      controller.setEventFilter('all');
      expect(controller.filteredEvents.length, equals(controller.events.length));

      // Clear events
      controller.clearEvents();
      expect(controller.events.isEmpty, isTrue);
      expect(controller.filteredEvents.isEmpty, isTrue);
    });
  });

  group('ShellScreen Navigation & Multi-Pane Widget Tests', () {
    testWidgets('Desktop layout (1024x768) renders NavigationRail and navigates between tabs',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1024, 768);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      final store = InMemorySettingsStore();
      final controller = PlatformController(settingsStore: store);

      await tester.pumpWidget(MiniAiApp(controller: controller));
      await tester.pumpAndSettle();

      // Desktop layout verification: NavigationRail present, bottom navigation bar absent
      expect(find.byKey(const Key('desktop_navigation_rail')), findsOneWidget);
      expect(find.byKey(const Key('mobile_navigation_bar')), findsNothing);

      // Active workspace badge in header
      expect(find.byKey(const Key('workspace_badge')), findsOneWidget);
      expect(find.text('default'), findsOneWidget);

      // Privacy status badge in header
      expect(find.byKey(const Key('privacy_status_badge')), findsOneWidget);
      expect(find.text('LOCAL-ONLY PRIVACY'), findsOneWidget);

      // Live Telemetry bar in header
      expect(find.byKey(const Key('telemetry_status_bar')), findsOneWidget);
      expect(find.textContaining('CPU 17%'), findsOneWidget);

      // Modality Studio is the default tab (index 0)
      expect(find.byKey(const Key('modality_card_llm')), findsOneWidget);

      // Navigate to Settings tab via workspace badge
      await tester.tap(find.byKey(const Key('workspace_badge')));
      await tester.pumpAndSettle();
      expect(controller.selectedNavigationIndex, equals(1));
      expect(find.text('Privacy & Cloud Governance'), findsOneWidget);
      expect(find.byKey(const Key('settings_workspace_input')), findsOneWidget);

      // Modify workspace in SettingsScreen
      await tester.enterText(find.byKey(const Key('settings_workspace_input')), 'science-lab-42');
      await tester.tap(find.byKey(const Key('settings_save_workspace_button')));
      await tester.pumpAndSettle();
      expect(controller.activeWorkspaceId, equals('science-lab-42'));
      expect(find.text('science-lab-42'), findsWidgets);

      // Navigate to Events tab
      await tester.tap(find.byKey(const Key('nav_rail_events')));
      await tester.pumpAndSettle();
      expect(controller.selectedNavigationIndex, equals(2));
      expect(find.byKey(const Key('event_list_view')), findsOneWidget);
      expect(find.byKey(const Key('emit_ping_icon_button')), findsOneWidget);

      // Test event filtering in EventInspectorView
      await tester.tap(find.byKey(const Key('filter_chip_workspace_switched')));
      await tester.pumpAndSettle();
      expect(controller.eventFilter, equals('workspace_switched'));

      // Navigate back to Studio tab
      await tester.tap(find.byKey(const Key('nav_rail_studio')));
      await tester.pumpAndSettle();
      expect(controller.selectedNavigationIndex, equals(0));
      expect(find.byKey(const Key('modality_card_llm')), findsOneWidget);
    });

    testWidgets('Mobile layout (400x800) renders NavigationBar and opens Live Event Drawer',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(400, 800);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      final store = InMemorySettingsStore();
      final controller = PlatformController(settingsStore: store);

      await tester.pumpWidget(MiniAiApp(controller: controller));
      await tester.pumpAndSettle();

      // Mobile layout verification: Bottom NavigationBar present, NavigationRail absent
      expect(find.byKey(const Key('mobile_navigation_bar')), findsOneWidget);
      expect(find.byKey(const Key('desktop_navigation_rail')), findsNothing);

      // Open Live Event Stream End Drawer via header button
      await tester.tap(find.byKey(const Key('open_event_drawer_button')));
      await tester.pumpAndSettle();

      // Verify drawer opened with event list
      expect(find.byKey(const Key('event_list_view')), findsOneWidget);

      // Emit diagnostic event from inside drawer toolbar
      await tester.tap(find.byKey(const Key('emit_ping_icon_button')));
      await tester.pumpAndSettle();
      expect(controller.events.first.eventType, equals('diagnostic_alert'));

      // Close drawer by tapping outside or pressing back
      await tester.tapAt(const Offset(20, 20));
      await tester.pumpAndSettle();
    });

    testWidgets('Settings persistence across controller instances', (WidgetTester tester) async {
      final store = InMemorySettingsStore();
      final controller1 = PlatformController(settingsStore: store);

      // Change settings on controller1
      controller1.setLocalOnlyMode(false);
      controller1.setForceCpuOnly(true);
      controller1.setActiveWorkspace('persistent-workspace');
      controller1.setIpcConfig(url: 'http://127.0.0.1:5555', nonce: 'nonce-secret');

      // Create controller2 using the same store and await loadSettings
      final controller2 = PlatformController(settingsStore: store);
      await controller2.loadSettings();

      expect(controller2.localOnlyMode, isFalse);
      expect(controller2.forceCpuOnly, isTrue);
      expect(controller2.activeWorkspaceId, equals('persistent-workspace'));
      expect(controller2.ipcServerUrl, equals('http://127.0.0.1:5555'));
      expect(controller2.sessionNonce, equals('nonce-secret'));
    });

    testWidgets('SettingsScreen buttons, toggles, consent controls, and diagnostics',
        (WidgetTester tester) async {
      final store = InMemorySettingsStore();
      final controller = PlatformController(settingsStore: store);

      // Render SettingsScreen directly
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SettingsScreen(controller: controller),
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Toggle Local-Only mode off and on
      await tester.tap(find.byKey(const Key('settings_local_only_switch')));
      await tester.pumpAndSettle();
      expect(controller.localOnlyMode, isFalse);

      // Grant consent and verify revocation button in SettingsScreen
      controller.grantCloudConsent(
        const CloudConsentToken(
          tokenId: 'settings-token-99',
          modality: Modality.llm,
          targetEndpoint: 'https://compute.miniai.local',
          maxPayloadBytes: 8192,
          issuedAtEpochSecs: 100,
          expiresAtEpochSecs: 200,
          userAcknowledged: true,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.textContaining('settings-token-99'), findsOneWidget);

      await tester.tap(find.text('Revoke'));
      await tester.pumpAndSettle();
      expect(controller.activeConsent, isNull);

      // Toggle force CPU switch
      await tester.tap(find.byKey(const Key('settings_force_cpu_switch')));
      await tester.pumpAndSettle();
      expect(controller.forceCpuOnly, isTrue);

      // Save IPC Config
      await tester.enterText(
        find.byKey(const Key('settings_ipc_url_input')),
        'http://127.0.0.1:4444',
      );
      await tester.enterText(
        find.byKey(const Key('settings_ipc_nonce_input')),
        'new-nonce',
      );
      await tester.tap(find.byKey(const Key('settings_save_ipc_button')));
      await tester.pumpAndSettle();
      expect(controller.ipcServerUrl, equals('http://127.0.0.1:4444'));
      expect(controller.sessionNonce, equals('new-nonce'));

      // Test connection (unreachable server fallback)
      await tester.tap(find.byKey(const Key('settings_test_ipc_button')));
      await tester.pumpAndSettle();
      expect(controller.ipcConnected, isFalse);

      // Reload settings button
      await tester.tap(find.byKey(const Key('settings_reload_button')));
      await tester.pumpAndSettle();

      // Emit diagnostic ping button
      await tester.tap(find.byKey(const Key('settings_diagnostic_ping_button')));
      await tester.pumpAndSettle();
      expect(controller.events.first.eventType, equals('diagnostic_alert'));
    });

    testWidgets('EventInspectorView inspects JSON payload dialog and handles empty state',
        (WidgetTester tester) async {
      final store = InMemorySettingsStore();
      final controller = PlatformController(settingsStore: store);

      // Clear events to test empty state
      controller.clearEvents();

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: EventInspectorView(controller: controller),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('No events recorded matching criteria.'), findsOneWidget);

      // Add diverse events with different types
      controller.addEvent(
        const EventEnvelope(
          sequence: 1,
          timestampUtc: '2026-11-05T01:00:00Z',
          eventType: 'privacy_policy_updated',
          payload: {'local_only_mode': true},
        ),
      );
      controller.addEvent(
        const EventEnvelope(
          sequence: 2,
          timestampUtc: '2026-11-05T01:01:00Z',
          eventType: 'hardware_changed',
          payload: {'backend': 'cpu'},
        ),
      );
      controller.addEvent(
        const EventEnvelope(
          sequence: 3,
          timestampUtc: '2026-11-05T01:02:00Z',
          eventType: 'adapter_health_changed',
          payload: {'adapter_id': 'vision-cpu-adapter'},
        ),
      );
      controller.addEvent(
        const EventEnvelope(
          sequence: 4,
          timestampUtc: '2026-11-05T01:03:00Z',
          eventType: 'unknown_event_type',
          payload: {'note': 'fallback test'},
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('event_list_view')), findsOneWidget);

      // Inspect JSON payload dialog
      await tester.tap(find.byKey(const Key('view_payload_button_4')));
      await tester.pumpAndSettle();

      expect(find.text('Event #4 (unknown_event_type)'), findsOneWidget);
      expect(find.byType(AlertDialog), findsOneWidget);

      // Close dialog
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      expect(find.text('Close'), findsNothing);

      // Clear events via button
      await tester.tap(find.byKey(const Key('clear_events_button')));
      await tester.pumpAndSettle();
      expect(find.text('No events recorded matching criteria.'), findsOneWidget);
    });

    testWidgets('DashboardScreen renders standalone mode with its own Scaffold & AppBar',
        (WidgetTester tester) async {
      final controller = PlatformController();
      await tester.pumpWidget(
        MaterialApp(
          home: DashboardScreen(controller: controller, embedded: false),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('MiniAI — Local-First Multimodal Platform'), findsOneWidget);
      expect(find.text('LOCAL-ONLY PRIVACY'), findsOneWidget);
    });

    test('IpcClient connects to real loopback server, authenticates, and handles offline',
        () async {
      HttpOverrides.global = null;
      final server = await HttpServer.bind('127.0.0.1', 0);
      final port = server.port;
      final baseUrl = 'http://127.0.0.1:$port';
      const nonce = 'session-token-xyz';

      server.listen((HttpRequest request) async {
        final auth = request.headers.value(HttpHeaders.authorizationHeader);
        if (request.uri.path == '/v1/health') {
          request.response.statusCode = HttpStatus.ok;
          request.response.write('{"status":"healthy"}');
          await request.response.close();
          return;
        }
        if (auth != 'Bearer $nonce') {
          request.response.statusCode = HttpStatus.unauthorized;
          await request.response.close();
          return;
        }
        if (request.uri.path == '/v1/system/snapshot') {
          request.response.statusCode = HttpStatus.ok;
          request.response.write('{"platform":"miniai","active_workspace":"test"}');
          await request.response.close();
          return;
        }
        if (request.uri.path == '/v1/events/recent') {
          request.response.statusCode = HttpStatus.ok;
          request.response.write(
            '[{"sequence":1,"timestamp_utc":"2026-01-01T00:00:00Z","event_type":"diagnostic_alert","payload":{"ping":true}}]',
          );
          await request.response.close();
          return;
        }
        if (request.uri.path == '/v1/events/stream') {
          request.response.statusCode = HttpStatus.ok;
          request.response.headers.contentType = ContentType('text', 'event-stream');
          request.response.write(
            'data: {"sequence":10,"timestamp_utc":"2026-01-01T00:00:00Z","event_type":"hardware_changed","payload":{"backend":"cpu"}}\n\n',
          );
          await request.response.flush();
          await request.response.close();
          return;
        }
        request.response.statusCode = HttpStatus.notFound;
        await request.response.close();
      });

      final client = IpcClient(baseUrl: baseUrl, sessionNonce: nonce);
      expect(await client.checkHealth(), isTrue);

      final snapshot = await client.fetchSystemSnapshot();
      expect(snapshot, isNotNull);
      expect(snapshot!['platform'], equals('miniai'));

      final recent = await client.fetchRecentEvents();
      expect(recent.length, equals(1));
      expect(recent.first.sequence, equals(1));

      final streamEvents = await client.connectEventStream().take(1).toList();
      expect(streamEvents.length, equals(1));
      expect(streamEvents.first.sequence, equals(10));
      expect(streamEvents.first.eventType, equals('hardware_changed'));

      await server.close(force: true);

      // Test offline / unreachable behavior
      final offlineClient = IpcClient(
        baseUrl: 'http://127.0.0.1:59999',
        sessionNonce: 'bad',
        timeout: const Duration(milliseconds: 100),
      );
      expect(await offlineClient.checkHealth(), isFalse);
      expect(await offlineClient.fetchSystemSnapshot(), isNull);
      expect(await offlineClient.fetchRecentEvents(), isEmpty);
      final offlineEvents = await offlineClient.connectEventStream().toList();
      expect(offlineEvents, isEmpty);
    });
  });
}
