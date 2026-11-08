import 'package:flutter_test/flutter_test.dart';
import 'package:miniai_app/src/controllers/platform_controller.dart';
import 'package:miniai_app/src/services/rust_backend_client.dart';

void main() {
  group('RustBackendClient Tests', () {
    late RustBackendClient client;

    setUp(() {
      client = RustBackendClient(
        daemonUrl: 'http://127.0.0.1:49999', // Non-running port to verify graceful fallback
        timeout: const Duration(milliseconds: 100),
      );
    });

    test('probes hardware with local fallback when daemon is offline', () async {
      final hw = await client.probeHardware();
      expect(hw, isNotEmpty);
      expect(hw.containsKey('logical_cpu_cores'), isTrue);
      expect(hw.containsKey('active_backend'), isTrue);
    });

    test('reads and sets privacy mode via client dispatch', () async {
      final p1 = await client.getPrivacyStatus();
      expect(p1['local_only_mode'], isTrue);

      final p2 = await client.setPrivacyMode(false);
      expect(p2['local_only_mode'], isFalse);
    });

    test('executes chat and knowledge search dispatch', () async {
      final chat = await client.executeChat(
        sessionId: 'session-test',
        prompt: 'what is adventure',
        modelId: 'llama-3.2-1b-instruct-q4',
      );
      expect(chat['status'], 'ok');

      final rag = await client.searchKnowledge('AdmissionGuard', limit: 3);
      expect(rag, isNotEmpty);
    });

    test('exercises listModels, memory operations, doctor, telemetry, and unloadModel', () async {
      final models = await client.listModels();
      expect(models, isEmpty);

      final memories = await client.listMemories();
      expect(memories.containsKey('semantic_facts'), isTrue);

      final stored = await client.storeMemory(
        memoryType: 'preference',
        content: 'Prefers dark mode',
        metadata: {'category': 'ui'},
      );
      expect(stored['id'], 'mem-local');

      final cleared = await client.clearMemories();
      expect(cleared['status'], 'cleared');

      final doctor = await client.runDoctor();
      expect(doctor['overall_status'], 'healthy');

      final telemetry = await client.getTelemetry();
      expect(telemetry.containsKey('total_chat_requests'), isTrue);

      final unloaded = await client.unloadModel(modelId: 'test-model');
      expect(unloaded['status'], 'unloaded');
    });

    test('PlatformController memory, verified download, and backendClient integration', () {
      final controller = PlatformController();
      expect(controller.backendClient, isNotNull);
      expect(controller.allowVerifiedModelDownloads, isFalse);

      controller.setAllowVerifiedModelDownloads(true);
      expect(controller.allowVerifiedModelDownloads, isTrue);

      final initialMemoriesCount = controller.memories.length;
      controller.storeMemory('fact', 'User resides in Paris');
      expect(controller.memories.length, initialMemoriesCount + 1);

      final addedMemory = controller.memories.last;
      expect(addedMemory.content, 'User resides in Paris');

      controller.deleteMemory(addedMemory.id);
      expect(controller.memories.length, initialMemoriesCount);

      controller.clearMemories();
      expect(controller.memories, isEmpty);

      controller.allowVerifiedModelDownload('llama-3.2-1b-instruct-q4');
      expect(controller.allowVerifiedModelDownloads, isTrue);
      controller.dispose();
    });

    test('RustBackendClient sessionNonce, daemon auto-discovery, and telemetry refresh', () async {
      expect(client.sessionNonce, isNotEmpty);
      expect(client.sessionNonce.startsWith('miniai_session_'), isTrue);

      final daemonRunning = await client.ensureDaemonRunning();
      expect(daemonRunning, isTrue);

      // Exercise real search path logic with forceCheck
      await client.ensureDaemonRunning(forceCheck: true);

      client.dispose();

      final controller = PlatformController();
      expect(controller.telemetryCpuPercent, 17);
      await controller.refreshTelemetry();
      expect(controller.telemetryCpuPercent, 17);
      expect(controller.peakMemoryRssMb, greaterThan(0.0));
      expect(controller.avgTokPerSec, greaterThan(0.0));
      controller.dispose();
    });
  });
}

