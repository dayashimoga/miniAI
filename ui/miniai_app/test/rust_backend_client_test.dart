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

    test('exercises all multimodal bridge methods and local fallbacks', () async {
      // 1. generateImage
      final img = await client.generateImage(
        prompt: 'Futuristic solar observatory',
        negativePrompt: 'foggy',
        mode: 'text_to_image',
        width: 256,
        height: 256,
        steps: 10,
        guidanceScale: 7.5,
        seed: 42,
      );
      expect(img['id'], isNotEmpty);
      expect(img['base64_png'], isNotEmpty);
      expect(img['sha256'], isNotEmpty);

      // 2. analyzeVision
      final vis = await client.analyzeVision(
        prompt: 'Detect bounding boxes',
        mode: 'ocr',
        width: 224,
        height: 224,
      );
      expect(vis['image_sha256'], isNotEmpty);
      expect(vis['ocr_boxes'], isNotEmpty);

      // 3. transcribeSpeech
      final asr = await client.transcribeSpeech(
        sampleRate: 16000,
        language: 'en',
      );
      expect(asr['text'], isNotEmpty);
      expect(asr['confidence'], greaterThan(0.9));

      // 4. synthesizeSpeech
      final tts = await client.synthesizeSpeech(
        text: 'Speech synthesis offline verification.',
        voiceId: 'alloy',
        pitch: 0.0,
        speed: 1.0,
        sampleRate: 22050,
      );
      expect(tts['audio_base64'], isNotEmpty);
      expect(tts['duration_secs'], greaterThan(0.0));

      // 5. synthesizeMusic
      final music = await client.synthesizeMusic(
        genre: 'Ambient acoustic strings',
        tempoBpm: 120,
        durationSecs: 4.0,
        keySignature: 'C Major',
        seed: 42,
      );
      expect(music['audio_base64'], isNotEmpty);
      expect(music['tempo_bpm'], 120);

      // 6. synthesizeVideo
      final video = await client.synthesizeVideo(
        prompt: 'Cinematic mountain sunrise drone shot',
        width: 320,
        height: 240,
        fps: 15.0,
        durationSecs: 2.0,
        seed: 42,
      );
      expect(video['video_id'], isNotEmpty);
      expect(video['sha256'], isNotEmpty);

      // 7. executeTrainingJob
      final train = await client.executeTrainingJob(
        taskType: 'lora_instruct',
        baseModel: 'llama-3.2-1b',
        datasetName: 'alpaca_clean',
        epochs: 3,
        learningRate: 0.0003,
        batchSize: 8,
        loraRank: 8,
      );
      expect(train['status'], 'completed');
      expect(train['history'], isNotEmpty);
    });

    test('Stage 20: Governor and KV-cache endpoints execute cleanly in client', () async {
      final client = RustBackendClient();

      // 1. getPowerProfile
      final p1 = await client.getPowerProfile();
      expect(p1['is_throttled'], isFalse);
      expect(p1['thermal_state'], 'nominal');
      expect(p1['max_context_tokens'], 4096);

      // 2. setPowerProfile (battery saving)
      final p2 = await client.setPowerProfile(batterySaving: true);
      expect(p2['is_throttled'], isTrue);
      expect(p2['battery_saving_active'], isTrue);
      expect(p2['max_context_tokens'], 1024);

      // 3. selectAcceleratorBackend
      final bCpu = await client.selectAcceleratorBackend('cpu');
      expect(bCpu['status'], 'selected');
      expect(bCpu['is_cpu'], isTrue);

      // 4. resetKvCache
      final rst = await client.resetKvCache();
      expect(rst['status'], 'kv_cache_reset');
    });
  });
}

