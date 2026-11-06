import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniai_app/src/controllers/platform_controller.dart';
import 'package:miniai_app/src/models/system_contracts.dart';
import 'package:miniai_app/src/screens/shell_screen.dart';
import 'package:miniai_app/src/screens/training_studio_screen.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Subphase 4.3.1 Training Studio Widget & Interaction Tests', () {
    late PlatformController controller;

    setUp(() {
      controller = PlatformController();
    });

    testWidgets('TrainingStudioScreen renders header, dataset splits, and hyperparameters',
        (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: TrainingStudioScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('training_studio_header_card')), findsOneWidget);
      expect(find.text('MiniAI Training Studio'), findsOneWidget);
      expect(find.byKey(const Key('training_device_badge')), findsOneWidget);
      expect(find.byKey(const Key('start_training_button')), findsOneWidget);

      // Dataset Split Card
      expect(find.byKey(const Key('dataset_split_card')), findsOneWidget);
      expect(find.byKey(const Key('dataset_selector_dropdown')), findsOneWidget);
      expect(find.text('Train: 960'), findsOneWidget);
      expect(find.text('Val: 120'), findsOneWidget);
      expect(find.text('Test: 120'), findsOneWidget);
      expect(find.byKey(const Key('dataset_sha_badge')), findsOneWidget);

      // Hyperparameters Form
      expect(find.byKey(const Key('hyperparameters_card')), findsOneWidget);
      expect(find.byKey(const Key('method_segmented_button')), findsOneWidget);
      expect(find.byKey(const Key('lora_rank_dropdown')), findsOneWidget);
      expect(find.byKey(const Key('lora_alpha_dropdown')), findsOneWidget);
      expect(find.byKey(const Key('epochs_dropdown')), findsOneWidget);
      expect(find.byKey(const Key('batch_size_dropdown')), findsOneWidget);
    });

    testWidgets('Changes dataset selection in dropdown and updates splits', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: TrainingStudioScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('dataset_selector_dropdown')));
      await tester.pumpAndSettle();

      await tester.tap(find.text('math-gsm8k-sample').last);
      await tester.pumpAndSettle();

      expect(controller.selectedDataset.name, equals('math-gsm8k-sample'));
      expect(find.text('Train: 640'), findsOneWidget);
      expect(find.text('Val: 80'), findsOneWidget);
      expect(find.text('Total: 800'), findsOneWidget);
    });

    testWidgets('Toggles training method and tunes hyperparameters', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: TrainingStudioScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      // Switch method to QLORA
      await tester.tap(find.text('QLORA'));
      await tester.pumpAndSettle();
      expect(controller.selectedTrainingMethod, equals(TrainingMethod.qlora));

      // Change rank
      await tester.tap(find.byKey(const Key('lora_rank_dropdown')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('r = 16').last);
      await tester.pumpAndSettle();
      expect(controller.trainingHyperparameters.rank, equals(16));

      // Change epochs
      await tester.tap(find.byKey(const Key('epochs_dropdown')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('1 epochs').last);
      await tester.pumpAndSettle();
      expect(controller.trainingHyperparameters.epochs, equals(1));
    });

    testWidgets('Executes training job to completion and registers adapter into catalog',
        (tester) async {
      // Set 1 epoch to speed up test execution
      controller.updateTrainingHyperparameters(
        controller.trainingHyperparameters.copyWith(epochs: 1),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: TrainingStudioScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      // Start fine-tuning
      await tester.tap(find.byKey(const Key('start_training_button')));
      await tester.pump();

      // Should show progress card
      expect(find.byKey(const Key('training_progress_card')), findsOneWidget);
      expect(find.byKey(const Key('training_progress_bar')), findsOneWidget);

      // Fast-forward until training completes
      while (controller.isTrainingActive) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      await tester.pumpAndSettle();

      // Verify completion state
      final job = controller.activeTrainingJob;
      expect(job, isNotNull);
      expect(job!.status, equals(TrainingJobStatus.completed));
      expect(find.byKey(const Key('job_status_badge')), findsOneWidget);
      expect(find.text('Completed & Certified'), findsOneWidget);
      expect(find.byKey(const Key('val_loss_badge')), findsOneWidget);
      expect(find.byKey(const Key('perplexity_badge')), findsOneWidget);
      expect(find.byKey(const Key('loss_history_view')), findsOneWidget);

      // Evaluation Comparison Card
      expect(find.byKey(const Key('evaluation_comparison_card')), findsOneWidget);
      expect(find.text('Base Model vs Fine-Tuned Adapter Comparison'), findsOneWidget);
      expect(find.byKey(const Key('register_adapter_button')), findsOneWidget);

      // Register into model catalog
      final initialCount = controller.models.length;
      await tester.ensureVisible(find.byKey(const Key('register_adapter_button')));
      await tester.tap(find.byKey(const Key('register_adapter_button')));
      await tester.pumpAndSettle();

      expect(controller.models.length, equals(initialCount + 1));
      expect(find.text('Adapter Registered in Catalog'), findsOneWidget);
    });

    testWidgets('Cancels training job when cancel button is clicked', (tester) async {
      controller.updateTrainingHyperparameters(
        controller.trainingHyperparameters.copyWith(epochs: 10),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: TrainingStudioScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('start_training_button')));
      await tester.pump();

      // Cancel button should appear
      expect(find.byKey(const Key('cancel_training_button')), findsOneWidget);
      await tester.tap(find.byKey(const Key('cancel_training_button')));
      await tester.pumpAndSettle();

      expect(controller.activeTrainingJob?.status, equals(TrainingJobStatus.cancelled));
      expect(find.text('Cancelled'), findsOneWidget);
    });

    testWidgets('ShellScreen navigates to Tab 6 Training Studio across desktop and mobile',
        (tester) async {
      // Desktop viewport (1024x768)
      tester.view.physicalSize = const Size(1024, 768);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      await tester.pumpWidget(
        MaterialApp(
          home: ShellScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      // Navigate to Train tab (index 6) via NavigationRail
      expect(find.byKey(const Key('nav_rail_training')), findsOneWidget);
      await tester.tap(find.byKey(const Key('nav_rail_training')));
      await tester.pumpAndSettle();

      expect(controller.selectedNavigationIndex, equals(6));
      expect(find.byKey(const Key('training_studio_header_card')), findsOneWidget);

      // Mobile viewport (400x800)
      tester.view.physicalSize = const Size(400, 800);
      await tester.pumpWidget(
        MaterialApp(
          home: ShellScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('mobile_navigation_bar')), findsOneWidget);
      expect(find.byKey(const Key('nav_bar_training')), findsOneWidget);
      await tester.tap(find.byKey(const Key('nav_bar_training')));
      await tester.pumpAndSettle();

      expect(controller.selectedNavigationIndex, equals(6));
    });
  });
}
