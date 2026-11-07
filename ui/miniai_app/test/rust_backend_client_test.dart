import 'package:flutter_test/flutter_test.dart';
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
      );
      expect(chat['status'], 'ok');

      final rag = await client.searchKnowledge('AdmissionGuard');
      expect(rag, isNotEmpty);
    });
  });
}

