import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniai_app/src/controllers/platform_controller.dart';
import 'package:miniai_app/src/models/system_contracts.dart';
import 'package:miniai_app/src/screens/hardware_inspector_screen.dart';
import 'package:miniai_app/src/screens/model_manager_screen.dart';
import 'package:miniai_app/src/screens/shell_screen.dart';
import 'package:miniai_app/src/services/settings_store.dart';

void main() {
  group('Subphase 2.3.2 Model Manager & Hardware Inspector Widget Tests', () {
    late PlatformController controller;

    setUp(() {
      controller = PlatformController(
        initialHardware: const HardwareProfile(
          osFamily: 'windows',
          arch: 'x86_64',
          logicalCpuCores: 16,
          totalMemoryBytes: 34359738368,
          availableMemoryBytes: 17179869184,
          detectedBackends: [AcceleratorBackend.cpu, AcceleratorBackend.vulkan],
          cpuFeatures: CpuFeatures(
            avx2: true,
            fma: true,
            sse4_2: true,
          ),
          gpus: [
            GpuDevice(
              name: 'Vulkan Test GPU',
              backend: AcceleratorBackend.vulkan,
              totalVramBytes: 8589934592,
              availableVramBytes: 6442450944,
            ),
          ],
        ),
        settingsStore: InMemorySettingsStore(),
        initialDownloadProgress: const {
          'whisper-tiny-en': DownloadProgressItem(
            modelId: 'whisper-tiny-en',
            bytesDownloaded: 77704715,
            totalBytes: 77704715,
            percentage: 100.0,
            status: DownloadStatus.verified,
          ),
        },
      );
    });

    testWidgets('ModelManagerScreen renders catalog, badges, and modality filtering',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1200, 1800);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: ModelManagerScreen(controller: controller),
        ),
      );

      // Verify header and model cards rendered
      expect(find.text('Model Registry & Verified Cache'), findsOneWidget);
      expect(find.byKey(const Key('model_card_llama-3.2-1b-instruct-q4')), findsOneWidget);
      expect(find.byKey(const Key('model_card_whisper-tiny-en')), findsOneWidget);

      // Verify SHA-256 verification badges
      expect(find.byKey(const Key('sha256_badge_llama-3.2-1b-instruct-q4')), findsOneWidget);
      expect(find.byKey(const Key('sha256_badge_whisper-tiny-en')), findsOneWidget);

      // Verify verified cache badge on whisper-tiny-en
      expect(find.text('Verified in Cache'), findsOneWidget);

      // Filter by LLM modality
      await tester.tap(find.byKey(const Key('filter_model_llm')));
      await tester.pumpAndSettle();

      expect(controller.modelModalityFilter, equals(Modality.llm));
      expect(find.byKey(const Key('model_card_llama-3.2-1b-instruct-q4')), findsOneWidget);
      expect(find.byKey(const Key('model_card_whisper-tiny-en')), findsNothing);

      // Filter back to all
      await tester.tap(find.byKey(const Key('filter_model_all')));
      await tester.pumpAndSettle();
      expect(controller.modelModalityFilter, isNull);
      expect(find.byKey(const Key('model_card_llama-3.2-1b-instruct-q4')), findsOneWidget);
    });

    testWidgets('Model download lifecycle: privacy blocking, pause, resume, and cancel',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1200, 1800);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: ModelManagerScreen(controller: controller),
        ),
      );

      // 1. In Local-Only Privacy Mode, remote download is blocked
      expect(controller.localOnlyMode, isTrue);
      await tester.tap(find.byKey(const Key('start_download_llama-3.2-1b-instruct-q4')));
      await tester.pumpAndSettle();

      expect(
        find.text('Remote downloads blocked in Local-Only Privacy Mode'),
        findsOneWidget,
      );
      expect(find.byKey(const Key('retry_download_llama-3.2-1b-instruct-q4')), findsOneWidget);

      // 2. Disable Local-Only Mode to permit download
      controller.setLocalOnlyMode(false);
      await tester.tap(find.byKey(const Key('retry_download_llama-3.2-1b-instruct-q4')));
      await tester.pumpAndSettle();

      // Progress bar should now appear
      expect(
        find.byKey(const Key('download_progress_bar_llama-3.2-1b-instruct-q4')),
        findsOneWidget,
      );
      expect(find.byKey(const Key('pause_download_llama-3.2-1b-instruct-q4')), findsOneWidget);

      // 3. Pause download
      await tester.tap(find.byKey(const Key('pause_download_llama-3.2-1b-instruct-q4')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('resume_download_llama-3.2-1b-instruct-q4')), findsOneWidget);

      // 4. Resume download
      await tester.tap(find.byKey(const Key('resume_download_llama-3.2-1b-instruct-q4')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('pause_download_llama-3.2-1b-instruct-q4')), findsOneWidget);

      // 5. Cancel download
      await tester.tap(find.byKey(const Key('cancel_download_llama-3.2-1b-instruct-q4')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('start_download_llama-3.2-1b-instruct-q4')), findsOneWidget);
    });

    testWidgets('Local Model Import Wizard opens dialog, registers model, and adds to catalog',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1200, 1800);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: ModelManagerScreen(controller: controller),
        ),
      );

      // Open import dialog
      await tester.tap(find.byKey(const Key('open_import_dialog_button')));
      await tester.pumpAndSettle();

      expect(find.text('Import Local Model Artifact'), findsOneWidget);

      // Fill in import form
      await tester.enterText(
        find.byKey(const Key('import_model_id_field')),
        'custom-llama-v1',
      );
      await tester.enterText(
        find.byKey(const Key('import_model_name_field')),
        'Custom Fine-Tuned LLaMA',
      );
      await tester.enterText(
        find.byKey(const Key('import_model_path_field')),
        'C:/models/custom_llama.gguf',
      );
      await tester.enterText(
        find.byKey(const Key('import_model_format_field')),
        'gguf',
      );

      // Confirm import
      await tester.tap(find.byKey(const Key('import_confirm_button')));
      await tester.pumpAndSettle();

      // Model now appears in catalog list as verified
      expect(find.byKey(const Key('model_card_custom-llama-v1')), findsOneWidget);
      expect(find.text('Custom Fine-Tuned LLaMA'), findsOneWidget);

      // Delete imported model
      await tester.tap(find.byKey(const Key('delete_model_custom-llama-v1')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('model_card_custom-llama-v1')), findsNothing);
    });

    testWidgets('HardwareInspectorScreen renders hardware profile, SIMD features, and admission guard',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1200, 1800);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: HardwareInspectorScreen(controller: controller),
        ),
      );

      expect(find.text('Hardware Capability & Admission Inspector'), findsOneWidget);
      expect(find.byKey(const Key('hardware_overview_card')), findsOneWidget);
      expect(find.byKey(const Key('detected_backend_cpu')), findsOneWidget);
      expect(find.byKey(const Key('detected_backend_vulkan')), findsOneWidget);

      // Verify SIMD chips
      expect(find.byKey(const Key('simd_feature_AVX2')), findsOneWidget);
      expect(find.byKey(const Key('simd_feature_FMA')), findsOneWidget);
      expect(find.byKey(const Key('simd_feature_SSE4.2')), findsOneWidget);

      // Verify 85% memory ceiling indicator
      expect(find.byKey(const Key('ram_ceiling_indicator')), findsOneWidget);
      expect(find.textContaining('85% Safe Admission Limit'), findsOneWidget);

      // Test memory admission input: 2048 MB -> should admit
      await tester.tap(find.byKey(const Key('test_admission_button')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('admission_result_banner')), findsOneWidget);
      expect(find.textContaining('ADMITTED'), findsOneWidget);

      // Test memory admission input: 50,000 MB -> should reject
      await tester.enterText(find.byKey(const Key('test_admission_input')), '50000');
      await tester.tap(find.byKey(const Key('test_admission_button')));
      await tester.pumpAndSettle();
      expect(find.textContaining('REJECTED'), findsOneWidget);
    });

    testWidgets('HardwareInspectorScreen renders live telemetry and unloads active model from RAM',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1200, 1800);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: HardwareInspectorScreen(controller: controller),
        ),
      );

      // Verify live telemetry card
      expect(find.byKey(const Key('live_telemetry_card')), findsOneWidget);
      expect(find.text('Live Resource Telemetry & Active Model'), findsOneWidget);
      expect(find.text('llama-3.2-1b-instruct-q4'), findsOneWidget);
      expect(find.byKey(const Key('unload_model_button')), findsOneWidget);
      expect(find.textContaining('Peak Memory:'), findsOneWidget);
      expect(find.textContaining('Avg TTFT:'), findsOneWidget);
      expect(find.textContaining('Throughput:'), findsOneWidget);

      // Unload active model from RAM
      await tester.tap(find.byKey(const Key('unload_model_button')));
      await tester.pumpAndSettle();

      // Verify model unloaded and memory freed
      expect(find.text('None (Memory Unloaded)'), findsOneWidget);
      expect(find.byKey(const Key('unload_model_button')), findsNothing);
      expect(find.textContaining('38.0 MB'), findsOneWidget);
    });

    testWidgets('ShellScreen navigates to Models (tab 3) and Hardware (tab 4)',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1024, 768);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: ShellScreen(controller: controller),
        ),
      );

      // Navigate to Models tab (index 3)
      await tester.tap(find.byKey(const Key('nav_rail_models')));
      await tester.pumpAndSettle();
      expect(controller.selectedNavigationIndex, equals(3));
      expect(find.text('Model Registry & Verified Cache'), findsOneWidget);

      // Navigate to Hardware tab (index 4)
      await tester.tap(find.byKey(const Key('nav_rail_hardware')));
      await tester.pumpAndSettle();
      expect(controller.selectedNavigationIndex, equals(4));
      expect(find.text('Hardware Capability & Admission Inspector'), findsOneWidget);
    });

    test('SystemContracts model registry and hardware serialization round-trips', () {
      // 1. CpuFeatures serialization
      const cpu = CpuFeatures(
        avx2: true,
        avx512f: true,
        fma: true,
        sse4_2: true,
        neon: true,
      );
      final cpuJson = cpu.toJson();
      final cpuRoundTrip = CpuFeatures.fromJson(cpuJson);
      expect(cpuRoundTrip.avx2, isTrue);
      expect(cpuRoundTrip.avx512f, isTrue);
      expect(cpuRoundTrip.enabledFeatures, contains('AVX2'));
      expect(cpuRoundTrip.enabledFeatures, contains('NEON'));

      // 2. GpuDevice serialization
      const gpu = GpuDevice(
        name: 'NVIDIA RTX 4090',
        backend: AcceleratorBackend.cuda,
        totalVramBytes: 24000000000,
        availableVramBytes: 20000000000,
      );
      final gpuJson = gpu.toJson();
      final gpuRoundTrip = GpuDevice.fromJson(gpuJson);
      expect(gpuRoundTrip.name, equals('NVIDIA RTX 4090'));
      expect(gpuRoundTrip.backend, equals(AcceleratorBackend.cuda));
      expect(gpuRoundTrip.safeVramLimitBytes, equals(17000000000));
      expect(gpuRoundTrip.canAdmitVram(10000000000), isTrue);
      expect(gpuRoundTrip.canAdmitVram(18000000000), isFalse);

      // 3. DownloadProgressItem serialization
      const progress = DownloadProgressItem(
        modelId: 'test-model',
        bytesDownloaded: 500,
        totalBytes: 1000,
        percentage: 50.0,
        status: DownloadStatus.downloading,
        errorMessage: null,
      );
      final progressJson = progress.toJson();
      final progressRoundTrip = DownloadProgressItem.fromJson(progressJson);
      expect(progressRoundTrip.modelId, equals('test-model'));
      expect(progressRoundTrip.percentage, equals(50.0));
      expect(progressRoundTrip.status, equals(DownloadStatus.downloading));

      // 4. ModelManifest serialization
      const manifest = ModelManifest(
        modelId: 'test-llama',
        name: 'Test LLaMA',
        modality: Modality.llm,
        version: '1.0.0',
        description: 'Test model',
        sourceUrl: 'http://127.0.0.1:4140/models/test.gguf',
        expectedSha256: 'abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789',
        expectedSizeBytes: 104857600, // 100 MB
        format: 'gguf',
        parameterCount: '1B',
        quantization: 'Q4_K_M',
        minRamBytes: 1073741824,
        recommendedBackend: AcceleratorBackend.cpu,
      );
      final manifestJson = manifest.toJson();
      final manifestRoundTrip = ModelManifest.fromJson(manifestJson);
      expect(manifestRoundTrip.modelId, equals('test-llama'));
      expect(manifestRoundTrip.shortSha256, equals('abcdef01...6789'));
      expect(manifestRoundTrip.formattedSize, equals('100.0 MB'));

      // 5. LocalModelImportRequest serialization
      const importReq = LocalModelImportRequest(
        sourcePath: 'C:/models/import.gguf',
        modelId: 'import-id',
        name: 'Import Name',
        modality: Modality.vision,
        version: '1.0.0',
        description: 'Import desc',
        format: 'gguf',
        parameterCount: '2B',
        quantization: 'FP16',
        minRamBytes: 2000000000,
        recommendedBackend: AcceleratorBackend.vulkan,
        expectedSha256: '0000000000000000000000000000000000000000000000000000000000000000',
        expectedSizeBytes: 5000000,
        overwrite: true,
      );
      final importJson = importReq.toJson();
      final importRoundTrip = LocalModelImportRequest.fromJson(importJson);
      expect(importRoundTrip.modelId, equals('import-id'));
      expect(importRoundTrip.modality, equals(Modality.vision));
      expect(importRoundTrip.overwrite, isTrue);
    });
  });
}
