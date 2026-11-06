import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniai_app/src/controllers/platform_controller.dart';
import 'package:miniai_app/src/models/system_contracts.dart';
import 'package:miniai_app/src/screens/shell_screen.dart';
import 'package:miniai_app/src/screens/vision_canvas_screen.dart';

void main() {
  group('Subphase 6.3.1 Vision & Canvas Studio Contract Tests', () {
    test('OcrBoxContract and VisionAnalysisResultContract JSON round-trip', () {
      const ocrBox = OcrBoxContract(
        text: 'MINIAI TEST',
        confidence: 0.99,
        x: 0.1,
        y: 0.2,
        width: 0.5,
        height: 0.15,
      );
      final boxJson = ocrBox.toJson();
      final decodedBox = OcrBoxContract.fromJson(boxJson);
      expect(decodedBox.text, equals('MINIAI TEST'));
      expect(decodedBox.confidence, equals(0.99));
      expect(decodedBox.x, equals(0.1));

      const result = VisionAnalysisResultContract(
        imageSha256: '9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08',
        width: 512,
        height: 512,
        format: 'PNG',
        description: 'Test visual scene with sharp contrast',
        tags: ['test', 'contrast', 'ocr'],
        ocrBoxes: [ocrBox],
        qaAnswer: 'Visual question answered.',
      );

      final resultJson = result.toJson();
      final decodedResult = VisionAnalysisResultContract.fromJson(resultJson);
      expect(decodedResult.imageSha256, equals(result.imageSha256));
      expect(decodedResult.width, equals(512));
      expect(decodedResult.format, equals('PNG'));
      expect(decodedResult.tags.length, equals(3));
      expect(decodedResult.ocrBoxes.length, equals(1));
      expect(decodedResult.qaAnswer, equals('Visual question answered.'));
    });

    test('GenerationModeContract wire values and display names', () {
      for (final mode in GenerationModeContract.values) {
        expect(GenerationModeContract.fromWire(mode.wireValue), equals(mode));
        expect(mode.displayName.isNotEmpty, isTrue);
      }
      expect(GenerationModeContract.fromWire('unknown'), equals(GenerationModeContract.textToImage));
    });

    test('ImageGenTaskConfig validation rules and copyWith', () {
      const validConfig = ImageGenTaskConfig(
        prompt: 'Futuristic solar observatory in desert',
        negativePrompt: 'blurry',
        width: 256,
        height: 512,
        steps: 25,
        guidanceScale: 8.0,
        seed: 12345,
        strength: 0.8,
        mode: GenerationModeContract.imageToImage,
      );

      expect(validConfig.isValid, isTrue);
      expect(validConfig.toJson()['width'], equals(256));
      expect(validConfig.toJson()['mode'], equals('image_to_image'));

      final decoded = ImageGenTaskConfig.fromJson(validConfig.toJson());
      expect(decoded.prompt, equals(validConfig.prompt));
      expect(decoded.steps, equals(25));
      expect(decoded.mode, equals(GenerationModeContract.imageToImage));

      // Invalid configs
      final emptyPrompt = validConfig.copyWith(prompt: '   ');
      expect(emptyPrompt.isValid, isFalse);

      final invalidWidth = validConfig.copyWith(width: 250); // not multiple of 64
      expect(invalidWidth.isValid, isFalse);

      final invalidSteps = validConfig.copyWith(steps: 0);
      expect(invalidSteps.isValid, isFalse);

      final invalidGuidance = validConfig.copyWith(guidanceScale: 0.5);
      expect(invalidGuidance.isValid, isFalse);

      final invalidStrength = validConfig.copyWith(strength: 1.5);
      expect(invalidStrength.isValid, isFalse);
    });

    test('GeneratedImageItem JSON round-trip and formatted size', () {
      const item = GeneratedImageItem(
        id: 'img-test-01',
        prompt: 'Aurora borealis over frozen lake',
        negativePrompt: 'lowres',
        mode: GenerationModeContract.textToImage,
        width: 512,
        height: 512,
        steps: 30,
        guidanceScale: 7.0,
        seed: 999,
        sha256: 'a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef',
        timestampEpochSecs: 1700000000,
        base64Png: 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==',
        provenanceSummary: 'Model: miniai-imagegen-cpu | Seed: 999',
        sizeBytes: 2500000,
      );

      expect(item.formattedSize, contains('MB'));
      const smallItem = GeneratedImageItem(
        id: 'img-test-02',
        prompt: 'Small',
        negativePrompt: '',
        mode: GenerationModeContract.textToImage,
        width: 64,
        height: 64,
        steps: 10,
        guidanceScale: 7.0,
        seed: 1,
        sha256: 'abc',
        timestampEpochSecs: 1700000000,
        base64Png: '',
        provenanceSummary: '',
        sizeBytes: 512,
      );
      expect(smallItem.formattedSize, equals('512 B'));

      final json = item.toJson();
      final decoded = GeneratedImageItem.fromJson(json);
      expect(decoded.id, equals('img-test-01'));
      expect(decoded.steps, equals(30));
      expect(decoded.sha256, equals(item.sha256));
    });
  });

  group('Subphase 6.3.1 Vision & Canvas Studio Widget Tests', () {
    late PlatformController controller;

    setUp(() {
      controller = PlatformController();
    });

    testWidgets('ShellScreen navigates to Tab 8 Canvas across Desktop layout', (tester) async {
      tester.view.physicalSize = const Size(1024, 768);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: ShellScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('desktop_navigation_rail')), findsOneWidget);
      expect(find.byKey(const Key('nav_rail_canvas')), findsOneWidget);

      await tester.tap(find.byKey(const Key('nav_rail_canvas')));
      await tester.pumpAndSettle();

      expect(controller.selectedNavigationIndex, equals(8));
      expect(find.byType(VisionCanvasScreen), findsOneWidget);
      expect(find.byKey(const Key('tab_vision_understanding')), findsOneWidget);
      expect(find.byKey(const Key('tab_canvas_studio')), findsOneWidget);
    });

    testWidgets('Vision Understanding tab: loads image, displays OCR, runs grounded Q&A, and rejects corrupt payloads', (tester) async {
      tester.view.physicalSize = const Size(1024, 768);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: VisionCanvasScreen(controller: controller),
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Verify initial upload card
      expect(find.byKey(const Key('vision_upload_card')), findsOneWidget);
      expect(find.byKey(const Key('vision_cpu_badge')), findsOneWidget);

      // Load sample diagram
      await tester.tap(find.byKey(const Key('load_sample_image_button')));
      await tester.pumpAndSettle();

      // Verify analysis viewer
      expect(find.byKey(const Key('vision_image_viewer')), findsOneWidget);
      expect(find.byKey(const Key('vision_format_badge')), findsOneWidget);
      expect(find.byKey(const Key('vision_dimensions_badge')), findsOneWidget);
      expect(find.byKey(const Key('vision_sha256_badge')), findsOneWidget);

      // Verify OCR boxes
      expect(find.byKey(const Key('ocr_result_card')), findsOneWidget);
      expect(find.byKey(const Key('ocr_box_0')), findsOneWidget);
      expect(find.text('MINIAI'), findsWidgets);

      // Verify description card and tags
      expect(find.byKey(const Key('vision_description_card')), findsOneWidget);
      expect(find.byKey(const Key('vision_tag_chip_local-first')), findsOneWidget);

      // Submit Grounded Visual QA question
      final qaSubmitBtn = find.byKey(const Key('vision_qa_submit_button'));
      await tester.ensureVisible(qaSubmitBtn);
      await tester.enterText(find.byKey(const Key('vision_qa_field')), 'What text is shown in the image?');
      await tester.tap(qaSubmitBtn);
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('vision_qa_answer_card')), findsOneWidget);
      expect(find.byKey(const Key('vision_qa_answer_text')), findsOneWidget);
      expect(find.textContaining('Detected text elements: MINIAI'), findsOneWidget);

      // Submit format question
      await tester.ensureVisible(qaSubmitBtn);
      await tester.enterText(find.byKey(const Key('vision_qa_field')), 'What is the image format?');
      await tester.tap(qaSubmitBtn);
      await tester.pumpAndSettle();
      expect(find.textContaining('The image format is PNG'), findsOneWidget);

      // Test corrupt image upload rejection
      final corruptBytes = Uint8List.fromList([0x00, 0x11, 0x22, 0x33, 0x44]);
      controller.uploadVisionImage(corruptBytes, 'corrupt.bin');
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('vision_error_banner')), findsOneWidget);
      expect(find.textContaining('Unsupported image format or invalid magic bytes'), findsOneWidget);
    });

    testWidgets('Canvas Studio tab: parameter controls, inpainting mask drawing, and gallery inspection', (tester) async {
      tester.view.physicalSize = const Size(1200, 800);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: VisionCanvasScreen(controller: controller),
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Switch to Canvas Studio tab
      await tester.tap(find.byKey(const Key('tab_canvas_studio')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('imagegen_controls_card')), findsOneWidget);
      expect(find.byKey(const Key('inpainting_canvas_card')), findsOneWidget);
      expect(find.byKey(const Key('canvas_gallery_view')), findsOneWidget);

      // Edit Prompt & Dimensions
      await tester.enterText(find.byKey(const Key('imagegen_prompt_field')), 'Cyberpunk neon rain cityscape');
      expect(controller.imageGenConfig.prompt, equals('Cyberpunk neon rain cityscape'));

      await tester.tap(find.byKey(const Key('dimension_chip_512')));
      await tester.pumpAndSettle();
      expect(controller.imageGenConfig.width, equals(512));
      expect(controller.imageGenConfig.height, equals(512));

      // Randomize seed
      await tester.tap(find.byKey(const Key('imagegen_random_seed_button')));
      await tester.pumpAndSettle();

      // Inpainting mask drawing & controls
      expect(find.byKey(const Key('inpainting_mask_canvas')), findsOneWidget);
      controller.addMaskPoint(const Offset(50, 50));
      controller.addMaskPoint(const Offset(60, 60));
      controller.setBrushSize(32.0);
      controller.setEraserMode(true);
      await tester.pumpAndSettle();

      expect(find.text('2 Points'), findsOneWidget);
      expect(controller.brushSize, equals(32.0));
      expect(controller.isEraserMode, isTrue);

      await tester.tap(find.byKey(const Key('clear_mask_button')));
      await tester.pumpAndSettle();
      expect(controller.maskPoints, isEmpty);

      // Gallery inspection
      expect(controller.galleryImages.length, greaterThanOrEqualTo(1));
      final sampleId = controller.galleryImages.first.id;

      final viewProvBtn = find.byKey(Key('view_provenance_button_$sampleId'));
      await tester.ensureVisible(viewProvBtn);
      await tester.tap(viewProvBtn);
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('image_provenance_sheet')), findsOneWidget);
      expect(find.byKey(const Key('provenance_sha256_text')), findsOneWidget);

      await tester.tap(find.byKey(const Key('close_provenance_dialog_button')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('image_provenance_sheet')), findsNothing);

      // Full synthesis execution
      final genBtn = find.byKey(const Key('generate_image_button'));
      await tester.ensureVisible(genBtn);
      await tester.tap(genBtn);
      await tester.pump(); // Start generation

      // Progress card active
      expect(controller.isGeneratingImage, isTrue);

      // Cancel generation
      final cancelBtn = find.byKey(const Key('cancel_imagegen_button'));
      await tester.ensureVisible(cancelBtn);
      await tester.tap(cancelBtn);
      await tester.pumpAndSettle();
      expect(controller.isGeneratingImage, isFalse);

      // Complete synthesis without ticks for instant assertion
      await controller.generateImage(simulateTicks: false);
      await tester.pumpAndSettle();

      expect(controller.galleryImages.length, greaterThanOrEqualTo(2));
      expect(controller.galleryImages.first.prompt, equals('Cyberpunk neon rain cityscape'));

      // Delete image
      final newlyGeneratedId = controller.galleryImages.first.id;
      final countBefore = controller.galleryImages.length;
      final delBtn = find.byKey(Key('delete_gallery_image_button_$newlyGeneratedId'));
      await tester.ensureVisible(delBtn);
      await tester.tap(delBtn);
      await tester.pumpAndSettle();
      expect(controller.galleryImages.length, equals(countBefore - 1));
    });
  });
}
