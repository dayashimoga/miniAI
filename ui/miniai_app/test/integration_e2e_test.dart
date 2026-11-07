import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniai_app/main.dart';
import 'package:miniai_app/src/controllers/platform_controller.dart';
import 'package:miniai_app/src/models/system_contracts.dart';
import 'package:miniai_app/src/screens/event_inspector_view.dart';
import 'package:miniai_app/src/services/ipc_client.dart';
import 'package:miniai_app/src/services/settings_store.dart';

void main() {
  group('Subphase 1.3.2 Cross-Layer Integration & Offline Resilience Tests', () {
    test('Live loopback IPC server syncs real-time events to PlatformController',
        () async {
      HttpOverrides.global = null;
      final server = await HttpServer.bind('127.0.0.1', 0);
      final port = server.port;
      final baseUrl = 'http://127.0.0.1:$port';
      const nonce = 'session-integration-nonce-42';

      final sseControllers = <StreamController<String>>[];

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
          request.response.write(
            jsonEncode({
              'hardware': {
                'os_family': 'windows',
                'arch': 'x86_64',
                'logical_cpu_cores': 8,
                'total_memory_bytes': 17179869184,
                'available_memory_bytes': 8589934592,
                'detected_backends': ['cpu'],
              },
              'privacy_policy': {
                'local_only_mode': true,
                'active_consent_token': null,
              },
              'adapter_health': [],
            }),
          );
          await request.response.close();
          return;
        }

        if (request.uri.path == '/v1/events/stream') {
          request.response.statusCode = HttpStatus.ok;
          request.response.headers.contentType = ContentType('text', 'event-stream');
          request.response.headers.set('Cache-Control', 'no-cache');
          request.response.bufferOutput = false;
          await request.response.flush();

          final sseController = StreamController<String>();
          sseControllers.add(sseController);

          final sub = sseController.stream.listen((eventPayload) async {
            request.response.write('data: $eventPayload\n\n');
            await request.response.flush();
          });

          request.response.done.then((_) {
            sub.cancel();
            sseControllers.remove(sseController);
          });
          return;
        }

        request.response.statusCode = HttpStatus.notFound;
        await request.response.close();
      });

      final client = IpcClient(
        baseUrl: baseUrl,
        sessionNonce: nonce,
        timeout: const Duration(seconds: 2),
      );
      final store = InMemorySettingsStore();
      final controller = PlatformController(
        settingsStore: store,
        ipcClient: client,
      );

      // Verify health check succeeds against loopback server
      final isHealthy = await controller.checkIpcHealth();
      expect(isHealthy, isTrue);
      expect(controller.ipcConnected, isTrue);

      final eventCompleter = Completer<EventEnvelope>();

      // Listen to SSE stream
      final sseSubscription = client.connectEventStream().listen((event) {
        controller.addEvent(event);
        if (!eventCompleter.isCompleted) {
          eventCompleter.complete(event);
        }
      });

      // Wait briefly for connection
      await Future<void>.delayed(const Duration(milliseconds: 100));

      // Emit event over SSE
      final testEventJson = jsonEncode({
        'sequence': 101,
        'timestamp_utc': '2026-11-05T13:00:00Z',
        'event_type': 'diagnostic_alert',
        'payload': {
          'message': 'Rust server telemetry alert: memory nominal',
          'metric': 'rss_ok',
        },
      });

      for (final sc in sseControllers) {
        sc.add(testEventJson);
      }

      // Await event arrival
      final receivedEvent = await eventCompleter.future.timeout(
        const Duration(seconds: 3),
      );

      expect(receivedEvent.sequence, equals(101));
      expect(receivedEvent.eventType, equals('diagnostic_alert'));
      expect(
        receivedEvent.payload['message'],
        equals('Rust server telemetry alert: memory nominal'),
      );
      expect(controller.events.any((e) => e.sequence == 101), isTrue);

      // Clean up server and stream
      await sseSubscription.cancel();
      for (final sc in sseControllers) {
        await sc.close();
      }
      await server.close(force: true);
    });

    testWidgets('Live Event Drawer renders received system events in UI',
        (WidgetTester tester) async {
      final store = InMemorySettingsStore();
      final controller = PlatformController(settingsStore: store);

      controller.addEvent(
        const EventEnvelope(
          sequence: 101,
          timestampUtc: '2026-11-05T13:00:00Z',
          eventType: 'diagnostic_alert',
          payload: {
            'message': 'Rust server telemetry alert: memory nominal',
            'metric': 'rss_ok',
          },
        ),
      );

      await tester.pumpWidget(MiniAiApp(controller: controller));
      await tester.pumpAndSettle();

      // Open event drawer
      await tester.tap(find.byKey(const Key('open_event_drawer_button')));
      await tester.pumpAndSettle();

      expect(find.textContaining('101'), findsOneWidget);
      expect(find.textContaining('memory nominal'), findsOneWidget);

      // Close drawer by tapping outside
      await tester.tapAt(const Offset(20, 20));
      await tester.pumpAndSettle();
    });

    testWidgets('EventInspectorView exports redacted diagnostics bundle',
        (WidgetTester tester) async {
      final store = InMemorySettingsStore();
      final controller = PlatformController(settingsStore: store);

      controller.addEvent(
        const EventEnvelope(
          sequence: 1,
          timestampUtc: '2026-11-05T13:00:00Z',
          eventType: 'diagnostic_alert',
          payload: {
            'message': 'Telemetry memory probe nominal',
          },
        ),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: EventInspectorView(controller: controller),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('export_diagnostics_button')), findsOneWidget);
      await tester.tap(find.byKey(const Key('export_diagnostics_button')));
      await tester.pumpAndSettle();

      expect(find.text('Redacted diagnostics bundle exported'), findsOneWidget);
      expect(
        controller.events.any((e) => e.eventType == 'diagnostic_bundle_exported'),
        isTrue,
      );
    });

    test('Offline resilience: functions completely disconnected from network',
        () async {
      final tempDir = await Directory.systemTemp.createTemp('miniai_offline_test_');
      final settingsFile = '${tempDir.path}/offline_settings.json';
      final fileStore = FileSettingsStore(filePath: settingsFile);

      // Configure unreachable IPC server to test offline resilience
      final offlineClient = IpcClient(
        baseUrl: 'http://127.0.0.1:59998',
        sessionNonce: 'offline-nonce',
        timeout: const Duration(milliseconds: 100),
      );

      final controller = PlatformController(
        settingsStore: fileStore,
        ipcClient: offlineClient,
      );

      // Verify IPC health check handles failure gracefully without crashing
      final isHealthy = await controller.checkIpcHealth();
      expect(isHealthy, isFalse);
      expect(controller.ipcConnected, isFalse);
      expect(controller.lastAuditMessage, contains('unavailable'));

      // Perform local operations offline
      controller.setActiveWorkspace('offline-isolated-workspace');
      controller.setForceCpuOnly(true);
      controller.emitDiagnosticPing('Offline local diagnostic ping');
      await controller.saveSettings();

      expect(controller.activeWorkspaceId, equals('offline-isolated-workspace'));
      expect(controller.forceCpuOnly, isTrue);
      expect(controller.events.first.eventType, equals('diagnostic_alert'));

      // Simulate full application restart by initializing a new controller from the file
      final restartedController = PlatformController(
        settingsStore: fileStore,
        ipcClient: offlineClient,
      );
      await restartedController.loadSettings();

      expect(
        restartedController.activeWorkspaceId,
        equals('offline-isolated-workspace'),
      );
      expect(restartedController.forceCpuOnly, isTrue);
      expect(restartedController.localOnlyMode, isTrue);

      await tempDir.delete(recursive: true);
    });
  });

  group('Subphase 1.3.2 Accessibility & Semantics Tests', () {
    testWidgets('Header and controls provide accessible Semantics annotations',
        (WidgetTester tester) async {
      final controller = PlatformController();
      await tester.pumpWidget(MiniAiApp(controller: controller));
      await tester.pumpAndSettle();

      // Check workspace badge semantics
      expect(
        find.bySemanticsLabel(RegExp('Active Workspace')),
        findsOneWidget,
      );

      // Check privacy badge semantics in Local-Only mode
      expect(
        find.bySemanticsLabel(RegExp('Local-Only Privacy Mode Active')),
        findsOneWidget,
      );

      // Toggle to Hybrid mode and check updated semantics
      controller.setLocalOnlyMode(false);
      await tester.pumpAndSettle();
      expect(
        find.bySemanticsLabel(RegExp('Hybrid Consent Mode')),
        findsOneWidget,
      );

      // Check Live Event Drawer button tooltip and key
      final drawerBtn = find.byKey(const Key('open_event_drawer_button'));
      expect(drawerBtn, findsOneWidget);
      final iconButton = tester.widget<IconButton>(drawerBtn);
      expect(iconButton.tooltip, equals('Open Live Event Stream Drawer'));

      // Verify all 7 modality cards have accessible semantics
      for (final modality in Modality.values) {
        final card = find.byKey(Key('modality_card_${modality.wireValue}'));
        expect(card, findsOneWidget);
      }
    });
  });
}
